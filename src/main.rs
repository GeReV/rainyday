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
use crate::render_gl::{ColorBuffer, Error, FrameBuffer, Program, Shader, Texture, Viewport};
use crate::vertex::Vertex;
use failure::err_msg;
use nalgebra as na;
use nalgebra::{Isometry2, Point2, Vector2, Vector3};
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
use rand::prelude::*;
use render_gl::buffer::*;
use resources::Resources;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

const MAX_DROPLET_COUNT: usize = 10_000;

const DROPLETS_PER_SECOND: usize = 50;

const VIEW_DISTANCE: f32 = 10.0;

const DROPLET_SIZE_GRAVITY_THRESHOLD: f32 = 5.0;
const PRIVATE_GRAVITY_FORCE_FACTOR_Y: f32 = 0.25;
const PRIVATE_GRAVITY_FORCE_FACTOR_X: f32 = 0.0;

const DROP_VERT: &str = include_str!("../assets/shaders/drop.vert");
const DROP_FRAG: &str = include_str!("../assets/shaders/drop.frag");
const DROP_WIPE_VERT: &str = include_str!("../assets/shaders/drop_wipe.vert");
const DROP_WIPE_FRAG: &str = include_str!("../assets/shaders/drop_wipe.frag");
const COLORED_QUAD_VERT: &str = include_str!("../assets/shaders/colored_quad.vert");
const COLORED_QUAD_FRAG: &str = include_str!("../assets/shaders/colored_quad.frag");
const QUAD_VERT: &str = include_str!("../assets/shaders/quad.vert");
const FINAL_FRAG: &str = include_str!("../assets/shaders/final.frag");

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

    let window_size = window.size();

    let mut viewport = Viewport::for_window(window_size.0 as i32, window_size.1 as i32);

    let view: na::Matrix4<f32> = (na::Translation3::<f32>::from(na::Point3::origin().coords)
        * na::Translation3::<f32>::from(na::Vector3::z() * VIEW_DISTANCE))
    .inverse()
    .to_homogeneous();

    let mut projection = na::Orthographic3::new(
        0.0,
        window_size.0 as f32,
        0.0,
        window_size.1 as f32,
        0.01,
        1000.0,
    );

    let matrix = projection.into_inner() * view;

    let background_texture = Texture::from_res_rgb("textures/background.jpg")
        .with_gen_mipmaps()
        .load(&gl, &res)?;

    let texture_rc = Rc::<Texture>::new(background_texture);

    let frame_buffer = FrameBuffer::new(&gl);

    let background =
        background::Background::new(&gl, texture_rc.clone(), window_size.0, window_size.1, 1.0)?;

    let quad = Quad::default(&gl);

    viewport.set_used(&gl);

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

    let drop_program = render_gl::Program::from_shaders(
        &gl,
        &[
            Shader::from_vert_source_str(&gl, DROP_VERT)?,
            Shader::from_frag_source_str(&gl, DROP_FRAG)?,
        ],
    )
    .map_err(|msg| Error::LinkError {
        message: msg,
        name: "drop".to_string(),
    })?;

    let drop_wipe_program = Program::from_shaders(
        &gl,
        &[
            Shader::from_vert_source_str(&gl, DROP_WIPE_VERT)?,
            Shader::from_frag_source_str(&gl, DROP_WIPE_FRAG)?,
        ],
    )
    .map_err(|msg| Error::LinkError {
        message: msg,
        name: "drop_wipe".to_string(),
    })?;

    let colored_quad_program = Program::from_shaders(
        &gl,
        &[
            Shader::from_vert_source_str(&gl, COLORED_QUAD_VERT)?,
            Shader::from_frag_source_str(&gl, COLORED_QUAD_FRAG)?,
        ],
    )
    .map_err(|msg| Error::LinkError {
        message: msg,
        name: "colored_quad".to_string(),
    })?;

    let final_program = Program::from_shaders(
        &gl,
        &[
            Shader::from_vert_source_str(&gl, QUAD_VERT)?,
            Shader::from_frag_source_str(&gl, FINAL_FRAG)?,
        ],
    )
    .map_err(|msg| Error::LinkError {
        message: msg,
        name: "final".to_string(),
    })?;

    let mut updates = Vec::<(CollisionObjectSlabHandle, CollisionObjectSlabHandle)>::new();

    let mut opened = true;

    let mut frames: VecDeque<f32> = VecDeque::with_capacity(100);

    let mut time_accumulator: f64 = 0.;
    let mut droplets_accumulator: usize = DROPLETS_PER_SECOND;

    let background_mask = Texture::new(&gl, window_size.0 as u32, window_size.1 as u32)?;

    let background_tex = Texture::new(&gl, window_size.0 as u32, window_size.1 as u32)?;

    let fullscreen_quad =
        Quad::new_with_size(&gl, 0.0, 0.0, window_size.1 as f32, window_size.0 as f32);

    let black = ColorBuffer::from_rgba(0.0, 0.0, 0.0, 1.0);

    {
        frame_buffer.bind();
        frame_buffer.attach_texture(&background_mask);

        black.set_used(&gl);
        black.clear(&gl);

        frame_buffer.unbind();
    }

    let mut resolution: Vector2<f32> = Vector2::new(viewport.w as f32, viewport.h as f32);

    let mut instant = Instant::now();
    'main: loop {
        let now = Instant::now();
        let delta = now.duration_since(instant);
        instant = now;

        time_accumulator += delta.as_secs_f64();

        if time_accumulator > 1.0 {
            time_accumulator -= 1.0;

            droplets_accumulator += DROPLETS_PER_SECOND;
        }

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

                    resolution = Vector2::new(viewport.w as f32, viewport.h as f32);
                }
                _ => {}
            }
        }

        imgui_sdl2.prepare_frame(imgui.io_mut(), &window, &event_pump.mouse_state());

        imgui.io_mut().delta_time = delta.as_secs_f32();

        let ui = imgui.frame();

        if frames.len() == 100 {
            frames.pop_front();
        }
        frames.push_back(ui.io().framerate);

        prepare_ui(
            &ui,
            &mut opened,
            &frames,
            droplets.used_count(),
            droplets_accumulator,
        );

        // Updates
        {
            gravity_non_linear(&mut droplets, &mut world, &mut rng, &delta);

            trail(
                &mut droplets,
                &mut world,
                &mut rng,
                &collision_group,
                &contacts_query,
                &delta,
            );

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
                    d.size = rng.gen_range(3.0, 8.0);

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
                    keep_droplet.size = ((keep_droplet.size * 0.5).powf(3.0)
                        + (delete_droplet_size * 0.5).powf(3.0))
                    .cbrt()
                        * 2.0;

                    keep.set_shape(ShapeHandle::new(Ball::new(keep_droplet.size * 0.5)));
                }
            }

            for (_, delete_handle) in updates.iter() {
                if let Some(delete) = world.collision_object(*delete_handle) {
                    droplets.free(*delete.data());
                    world.remove(&[*delete_handle]);
                }
            }
        }

        // Background pass
        {
            background.prepass(&gl, &view, &matrix, &resolution);

            frame_buffer.bind();
            frame_buffer.attach_texture(&background_tex);

            background.render(&gl, &view, &matrix, &resolution);

            frame_buffer.unbind();
        }

        // Mask pass
        {
            frame_buffer.bind();
            frame_buffer.attach_texture(&background_mask);

            unsafe {
                gl.BlendFuncSeparate(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA, gl::ZERO, gl::ONE);
            }

            {
                colored_quad_program.set_used();

                if let Some(loc) = colored_quad_program.get_uniform_location("MVP") {
                    colored_quad_program.set_uniform_matrix_4fv(loc, &matrix);
                }

                if let Some(loc) = colored_quad_program.get_uniform_location("Color") {
                    colored_quad_program.set_uniform_4f(
                        loc,
                        &na::Vector4::new(0.0, 0.0, 0.0, 0.25 * delta.as_secs_f32()),
                    );
                }

                fullscreen_quad.render(&gl);
            }

            {
                let program_matrix_location = drop_wipe_program.get_uniform_location("MVP");
                let resolution_location = drop_wipe_program.get_uniform_location("Resolution");

                drop_wipe_program.set_used();

                if let Some(loc) = resolution_location {
                    drop_wipe_program.set_uniform_2f(loc, &resolution);
                }

                if let Some(loc) = program_matrix_location {
                    drop_wipe_program.set_uniform_matrix_4fv(loc, &matrix);
                }

                render_droplets(&gl, &quad, &droplets);
            }

            unsafe {
                gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
            }

            frame_buffer.unbind();
        }

        // Merge pass
        {
            final_program.set_used();

            black.set_used(&gl);
            black.clear(&gl);

            if let Some(loc) = final_program.get_uniform_location("MVP") {
                final_program.set_uniform_matrix_4fv(loc, &matrix);
            }

            if let Some(loc) = final_program.get_uniform_location("Texture0") {
                background_tex.bind_at(0);
                final_program.set_uniform_1i(loc, 0);
            }
            if let Some(loc) = final_program.get_uniform_location("Texture1") {
                texture_rc.bind_at(1);
                final_program.set_uniform_1i(loc, 1);
            }
            if let Some(loc) = final_program.get_uniform_location("Mask") {
                background_mask.bind_at(2);
                final_program.set_uniform_1i(loc, 2);
            }

            fullscreen_quad.render(&gl);
        }

        {
            drop_program.set_used();

            if let Some(loc) = drop_program.get_uniform_location("Resolution") {
                drop_program.set_uniform_2f(loc, &resolution);
            }

            if let Some(loc) = drop_program.get_uniform_location("MVP") {
                drop_program.set_uniform_matrix_4fv(loc, &matrix);
            }

            if let Some(loc) = drop_program.get_uniform_location("Texture") {
                texture_rc.bind_at(0);
                drop_program.set_uniform_1i(loc, 0);
            }

            render_droplets(&gl, &quad, &droplets);
        }

        imgui_sdl2.prepare_render(&ui, &window);
        renderer.render(ui);

        window.gl_swap_window();
    }

    Ok(())
}

fn prepare_ui(
    ui: &imgui::Ui,
    opened: &mut bool,
    frames: &VecDeque<f32>,
    droplets_used_count: usize,
    droplets_accumulator: usize,
) {
    let w = imgui::Window::new(imgui::im_str!("FPS"))
        .opened(opened)
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
        ui.text(&imgui::im_str!("Drops: {}", droplets_used_count));
        ui.text(&imgui::im_str!("Drops budget: {}", droplets_accumulator));
    });
}

fn render_droplets(gl: &gl::Gl, quad: &Quad, droplets: &Droplets) {
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

fn gravity_non_linear(
    droplets: &mut Droplets,
    world: &mut CollisionWorld<f32, usize>,
    rng: &mut ThreadRng,
    dt: &Duration,
) {
    let fps = 1.0 / dt.as_secs_f32();
    let gravity_y = PRIVATE_GRAVITY_FORCE_FACTOR_Y * dt.as_secs_f32();

    for i in 0..droplets.len() {
        let mut delete_index: Option<usize> = None;

        {
            let droplet = &mut droplets[i];

            if droplet.deleted || droplet.size < DROPLET_SIZE_GRAVITY_THRESHOLD {
                continue;
            }

            if droplet.size < DROPLET_SIZE_GRAVITY_THRESHOLD && droplet.seed > 0 {
                droplet.slowing = true;
            }

            let movement_probability = 0.01 * dt.as_secs_f64();

            if droplet.seed <= 0 {
                droplet.seed = (droplet.size * 0.5 * rng.gen_range(0.0, 1.0) * fps).floor() as i32;
                droplet.skipping = droplet.skipping == false;
                droplet.slowing = true;
            }

            droplet.seed -= 1;

            assert!(droplet.size >= 1.0);

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
            } else if rng.gen_bool((1.0 - 1.0 / droplet.size as f64) * movement_probability) {
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

fn trail(
    droplets: &mut Droplets,
    world: &mut CollisionWorld<f32, usize>,
    rng: &mut ThreadRng,
    collision_group: &CollisionGroups,
    contacts_query: &GeometricQueryType<f32>,
    dt: &Duration,
) {
    let gravity_y = PRIVATE_GRAVITY_FORCE_FACTOR_Y * dt.as_secs_f32();

    for i in 0..droplets.len() {
        let pos;
        let size;

        {
            let droplet = &mut droplets[i];

            if droplet.speed.y <= gravity_y {
                continue;
            }

            if droplet.size >= 6.0
                && (droplet.last_trail_y.is_none()
                    || (droplet.last_trail_y.unwrap_or(0.0) - droplet.pos.y)
                        >= rng.gen_range(0.1, 1.0) * 200.0)
            {
                droplet.last_trail_y = Some(droplet.pos.y);

                size = rng.gen_range(0.9, 1.1) * droplet.size * 0.25;
                pos = Vector2::new(
                    droplet.pos.x + rng.gen_range(-1.0, 1.0),
                    droplet.pos.y + droplet.size * 0.5 + droplet.speed.y + size * 0.5,
                );

                droplet.size =
                    ((droplet.size * 0.5).powf(3.0) - (size * 0.5).powf(3.0)).cbrt() * 2.0;

                if let Some(droplet_collision) = world.get_mut(droplet.collision_handle) {
                    droplet_collision.set_shape(ShapeHandle::new(Ball::new(droplet.size * 0.5)))
                }
            } else {
                continue;
            }
        }

        if let Some((i, d)) = droplets.checkout() {
            d.pos = pos;
            d.size = size;

            let shape_handle = ShapeHandle::new(Ball::new(d.size * 0.5));

            let handle = world
                .add(
                    Isometry2::new(d.pos.clone_owned(), na::zero()),
                    shape_handle,
                    *collision_group,
                    *contacts_query,
                    i,
                )
                .0;

            d.collision_handle = handle;
        }
    }
}
