extern crate gl;
extern crate sdl2;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate render_gl_derive;
extern crate nalgebra;
extern crate ncollide2d;
extern crate rand;

mod background;
mod debug;
mod droplet;
mod droplets;
mod quad;
pub mod render_gl;
pub mod resources;
mod vertex;

use crate::debug::failure_to_string;
use crate::droplet::Droplet;
use crate::droplets::Droplets;
use crate::quad::Quad;
use crate::render_gl::{Program, Texture};
use crate::vertex::Vertex;
use failure::err_msg;
use nalgebra as na;
use nalgebra::{Isometry2, Point2, Vector2};
use ncollide2d as nc;
use ncollide2d::bounding_volume::BoundingSphere;
use ncollide2d::broad_phase::{BroadPhase, DBVTBroadPhase};
use ncollide2d::pipeline::{
    default_narrow_phase, BallBallManifoldGenerator, BallBallProximityDetector, CollisionGroups,
    CollisionObject, CollisionObjectSlabHandle, CollisionWorld, ContactEvent, GeometricQueryType,
    NarrowPhase,
};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, ShapeHandle};
use rand::Rng;
use render_gl::buffer::*;
use resources::Resources;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

const MAX_DROPLET_COUNT: usize = 10_000;

const DROPLETS_PER_SECOND: usize = 50;

fn main() {
    if let Err(e) = run() {
        println!("{}", failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {
    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

    let sdl = sdl2::init().map_err(err_msg)?;

    let video_subsystem = sdl.video().map_err(err_msg)?;

    let gl_attr = video_subsystem.gl_attr();

    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let initial_window_size: (i32, i32) = (1920, 1080);

    let window = video_subsystem
        .window(
            "Rain",
            initial_window_size.0 as u32,
            initial_window_size.1 as u32,
        )
        .position_centered() //        .fullscreen_desktop()
        .opengl()
        .resizable()
        .allow_highdpi()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;

    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    let mut imgui = imgui::Context::create();
    imgui.set_ini_filename(None);

    let mut imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui, &window);

    let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui, |s| {
        video_subsystem.gl_get_proc_address(s) as _
    });

    let mut viewport =
        render_gl::Viewport::for_window(initial_window_size.0, initial_window_size.1);

    let distance = 10.0;

    let view: na::Matrix4<f32> = (na::Translation3::<f32>::from(na::Point3::origin().coords)
        * na::Translation3::<f32>::from(na::Vector3::z() * distance))
    .inverse()
    .to_homogeneous();

    let mut projection = na::Orthographic3::new(
        0.0,
        initial_window_size.0 as f32,
        0.0,
        initial_window_size.1 as f32,
        0.01,
        1000.0,
    );

    let matrix = projection.into_inner() * view;

    let texture = render_gl::Texture::from_res_rgb("textures/background.jpg")
        .with_gen_mipmaps()
        .load(&gl, &res)?;

    let texture_rc = Rc::<render_gl::Texture>::new(texture);

    let texture_buffer = render_gl::Texture::new(
        &gl,
        initial_window_size.0 as u32,
        initial_window_size.1 as u32,
    )?;

    let frame_buffer = render_gl::FrameBuffer::new(&gl);

    frame_buffer.bind();
    frame_buffer.attach_texture(&texture_buffer);

    let color_buffer2 = render_gl::ColorBuffer::from_color(na::Vector3::new(0.5, 0.6, 0.8));

    color_buffer2.set_used(&gl);
    color_buffer2.clear(&gl);

    frame_buffer.unbind();

    let background = background::Background::new(
        &res,
        &gl,
        texture_rc.clone(),
        initial_window_size.0 as u32,
        initial_window_size.1 as u32,
    )?;

    let quad = Quad::new(&gl)?;

    viewport.set_used(&gl);

    let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));

    color_buffer.set_used(&gl);

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;

    let mut rng = rand::thread_rng();

    let mut droplets: Droplets = Droplets::with_capacity(MAX_DROPLET_COUNT);

    let mut world = CollisionWorld::new(2.0);

    let collision_group = CollisionGroups::new();
    let contacts_query = GeometricQueryType::Proximity(0.0);

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut instant = Instant::now();

    let program = render_gl::Program::from_res(&gl, &res, "shaders/drop")?;

    let mut updates = Vec::<(CollisionObjectSlabHandle, CollisionObjectSlabHandle)>::new();

    let mut opened = true;

    let mut frames: VecDeque<f32> = VecDeque::with_capacity(100);

    let mut time_accumulator: f64 = 0.;
    let mut droplets_accumulator: usize = DROPLETS_PER_SECOND;

    'main: loop {
        for event in event_pump.poll_iter() {
            imgui_sdl2.handle_event(&mut imgui, &event);
            if imgui_sdl2.ignore_event(&event) {
                continue;
            }

            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::KeyUp { .. } => break 'main,
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w, h);
                    viewport.set_used(&gl);

                    projection.set_left_and_right(0.0, w as f32);
                    projection.set_bottom_and_top(0.0, h as f32);
                }
                _ => {}
            }
        }

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        let now = Instant::now();
        let delta = now.duration_since(instant);
        instant = now;

        imgui.io_mut().delta_time = delta.as_secs_f32();

        let ui = imgui.frame();

        {
            let temp_frames = &mut frames;

            if temp_frames.len() == 100 {
                temp_frames.pop_front();
            }
            temp_frames.push_back(ui.io().framerate);
        }

        let w = imgui::Window::new(imgui::im_str!("FPS"))
            .opened(&mut opened)
            .position([20.0, 20.0], imgui::Condition::Appearing)
            .always_auto_resize(true);
        w.build(&ui, || {
            let values = frames.iter().copied().collect::<Vec<f32>>();

            ui.text(&imgui::im_str!(
                "FPS: {:.1} ({:.1}ms)",
                ui.io().framerate,
                ui.io().delta_time * 1000.0
            ));
            imgui::PlotHistogram::new(&ui, imgui::im_str!(""), &values)
                .scale_max(150.0)
                .scale_min(0.0)
                .graph_size([220.0, 60.0])
                .build();
        });

        let resolution: na::Vector2<f32> =
            na::Vector2::<f32>::new(viewport.w as f32, viewport.h as f32);

        gravity_non_linear(&mut droplets, &mut world, &delta);

        updates.clear();

        // We get an "allowance" of DROPLETS_PER_SECOND every second.
        // This part of the loop will attempt to spend them at random times, and is more likely to
        // spend them the more time has past.
        // TODO: Any better way to spend these more evenly?
        // TODO: What happens when budget > fps?
        if droplets_accumulator > 0 && rng.gen_bool(time_accumulator.max(0.0).min(1.0)) {
            if let Some((i, d)) = droplets.checkout() {
                d.pos = na::Vector2::new(
                    rng.gen_range(0.0, viewport.w as f32),
                    rng.gen_range(0.0, viewport.h as f32),
                );
                d.size = rng.gen_range(1.5, 7.0);

                let shape_handle = ShapeHandle::new(Ball::new(d.size * 0.5));

                let handle = world
                    .add(
                        Isometry2::new(d.pos.clone_owned(), na::zero()),
                        shape_handle,
                        collision_group,
                        contacts_query,
                        i,
                    )
                    .0;

                d.collision_handle = handle;

                droplets_accumulator -= 1;
            }
        }

        for ev in world.proximity_events().iter().collect::<Vec<_>>() {
            if ev.new_status == Proximity::Intersecting {
                if let (Some(obj1), Some(obj2)) = (
                    world.collision_object(ev.collider1),
                    world.collision_object(ev.collider2),
                ) {
                    let sphere1 = obj1.shape().local_bounding_sphere();
                    let sphere2 = obj2.shape().local_bounding_sphere();

                    let rad1 = sphere1.radius();
                    let rad2 = sphere2.radius();

                    let pair = if rad1 > rad2 {
                        (ev.collider1, ev.collider2)
                    } else if rad1 < rad2 {
                        (ev.collider2, ev.collider1)
                    } else if sphere1.center().y > sphere2.center().y {
                        (ev.collider1, ev.collider2)
                    } else {
                        (ev.collider2, ev.collider1)
                    };

                    updates.push(pair);
                }
            }
        }

        for (keep_handle, delete_handle) in updates.iter() {
            if let (Some(keep), Some(delete)) =
                world.collision_object_pair_mut(*keep_handle, *delete_handle)
            {
                let keep_droplet_index = *keep.data();
                let delete_droplet_index = *delete.data();

                let delete_droplet_size = droplets[delete_droplet_index].size;

                let keep_droplet = &mut droplets[keep_droplet_index];

                // TODO: How much does a droplet grow when is absorbs another?
                keep_droplet.size += delete_droplet_size.cbrt() * 0.5;

                keep.set_shape(ShapeHandle::new(Ball::new(keep_droplet.size * 0.5)));
            }
        }

        for (_, delete_handle) in updates.iter() {
            if let Some(delete) = world.collision_object(*delete_handle) {
                droplets.free(*delete.data());
                world.remove(&[*delete_handle]);
            }
        }

        color_buffer.clear(&gl);

        background.render(&gl, 1.0, &view, &matrix, &resolution);

        render_droplets(
            &gl,
            &program,
            &matrix,
            &resolution,
            texture_rc.clone(),
            &quad,
            &droplets,
        );

        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(ui);

        window.gl_swap_window();

        time_accumulator += delta.as_secs_f64();

        if time_accumulator > 1.0 {
            time_accumulator -= 1.0;

            droplets_accumulator += DROPLETS_PER_SECOND;
        }
    }

    Ok(())
}

fn render_droplets(
    gl: &gl::Gl,
    program: &Program,
    matrix: &na::Matrix4<f32>,
    resolution: &na::Vector2<f32>,
    texture: Rc<Texture>,
    quad: &Quad,
    droplets: &Droplets,
) {
    let program_matrix_location = program.get_uniform_location("MVP");
    let texture_location = program.get_uniform_location("Texture");
    let resolution_location = program.get_uniform_location("Resolution");

    program.set_used();

    if let Some(loc) = resolution_location {
        program.set_uniform_2f(loc, &resolution);
    }

    if let Some(loc) = program_matrix_location {
        program.set_uniform_matrix_4fv(loc, &matrix);
    }

    if let Some(loc) = texture_location {
        texture.bind_at(0);
        program.set_uniform_1i(loc, 0);
    }

    quad.vao.bind();

    let instance_vbo: ArrayBuffer = ArrayBuffer::new(&gl);
    instance_vbo.bind();

    let offsets: Vec<na::Vector3<f32>> = droplets
        .into_iter()
        .filter(|d| !d.deleted)
        .map(|d| na::Vector3::new(d.pos.x, d.pos.y, d.size))
        .collect();

    instance_vbo.static_draw_data(&offsets);

    unsafe {
        gl.EnableVertexAttribArray(3);
        gl.VertexAttribPointer(
            3,
            3,         // the number of components per generic vertex attribute
            gl::FLOAT, // data type
            gl::FALSE,
            std::mem::size_of::<na::Vector3<f32>>() as gl::types::GLint,
            0 as *const gl::types::GLvoid,
        );
    }
    instance_vbo.unbind();

    unsafe {
        gl.VertexAttribDivisor(3, 1);
    }

    unsafe {
        gl.DrawElementsInstanced(
            gl::TRIANGLES,
            6,
            gl::UNSIGNED_BYTE,
            ::std::ptr::null(),
            offsets.len() as i32,
        );
    }
    quad.vao.unbind();
}

const PRIVATE_GRAVITY_FORCE_FACTOR_Y: f32 = 0.2;
const PRIVATE_GRAVITY_FORCE_FACTOR_X: f32 = 0.0;

fn gravity_non_linear(
    droplets: &mut Droplets,
    world: &mut CollisionWorld<f32, usize>,
    dt: &Duration,
) {
    let mut rng = rand::thread_rng();

    let gravity_y = PRIVATE_GRAVITY_FORCE_FACTOR_Y * dt.as_secs_f32();

    for i in 0..droplets.len() {
        let mut delete_index: Option<usize> = None;

        {
            let droplet = &mut droplets[i];

            if droplet.deleted {
                continue;
            }

            if droplet.seed <= 0 {
                droplet.seed = (droplet.size * rng.gen::<f32>() * 100.0).floor() as i32;
                droplet.skipping = droplet.skipping == false;
                droplet.slowing = true;
            }

            droplet.seed -= 1;

            if droplet.speed.y > 0.0 {
                if droplet.slowing {
                    droplet.speed *= 0.9;
                    if droplet.speed.y < gravity_y {
                        droplet.slowing = false;
                    }
                } else if droplet.skipping {
                    droplet.speed.y = gravity_y;
                    droplet.speed.x = PRIVATE_GRAVITY_FORCE_FACTOR_X;
                } else {
                    droplet.speed.y += gravity_y * droplet.size;
                    droplet.speed.x += PRIVATE_GRAVITY_FORCE_FACTOR_X * droplet.size;
                }
            } else if droplet.seed >= (95.0 * droplet.size) as i32 {
                droplet.speed.y = gravity_y;
                droplet.speed.x = PRIVATE_GRAVITY_FORCE_FACTOR_X;
            }

            //        if this.options.gravityAngleVariance != 0 {
            //            droplet.x_speed +=
            //                (rnd.gen() * 2 - 1) * droplet.y_speed * this.options.gravityAngleVariance
            //        }

            droplet.pos.y -= droplet.speed.y;
            droplet.pos.x += droplet.speed.x;

            if droplet.pos.y + droplet.size * 0.5 < 0.0 {
                delete_index = Some(i);

                world.remove(&[droplet.collision_handle]);
            } else if droplet.speed.x != 0.0 || droplet.speed.y != 0.0 {
                let handle = droplet.collision_handle;

                let object = world.get_mut(handle).unwrap();

                object.set_position(Isometry2::new(droplet.pos.clone_owned(), na::zero()));
            }
        }

        if let Some(delete_index) = delete_index {
            droplets.free(delete_index);
        }
    }

    world.update();
}
