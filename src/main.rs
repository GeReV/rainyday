extern crate gl;
extern crate sdl2;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate render_gl_derive;
extern crate nalgebra;
extern crate rand;
extern crate vec_2_10_10_10;

mod background;
mod debug;
mod drop_quad;
mod droplet;
mod quad;
pub mod render_gl;
pub mod resources;

use crate::debug::failure_to_string;
use crate::droplet::Droplet;
use crate::quad::Quad;
use failure::err_msg;
use nalgebra as na;
use rand::Rng;
use render_gl::buffer;
use render_gl::data;
use resources::Resources;
use sdl2::init;
use std::ffi::{CStr, CString};
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

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
        //        .fullscreen_desktop()
        .opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;

    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    let mut viewport =
        render_gl::Viewport::for_window(initial_window_size.0, initial_window_size.1);

    let distance = 10.0;

    let view = (na::Translation3::<f32>::from(na::Point3::origin().coords)
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

    let texture = render_gl::Texture::from_res_rgb("textures/background.jpg")
        .with_gen_mipmaps()
        .load(&gl, &res)?;

    let texture_rc = Rc::<render_gl::Texture>::new(texture);

    let background = background::Background::new(
        &res,
        &gl,
        texture_rc.clone(),
        initial_window_size.0 as u32,
        initial_window_size.1 as u32,
    )?;

    let droplet = drop_quad::DropQuad::new(&res, &gl, texture_rc.clone())?;

    viewport.set_used(&gl);

    let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));

    color_buffer.set_used(&gl);

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;

    let mut rng = rand::thread_rng();

    let mut droplets: Vec<Droplet> = (0..600)
        .map(|_| {
            let x = rng.gen_range(0.0, viewport.w as f32);
            let y = rng.gen_range(0.0, viewport.h as f32);

            let size = rng.gen_range(1.0, 5.0);

            Droplet {
                x,
                y,
                size,
                collided: false,
                seed: 0,
                skipping: false,
                slowing: false,
                x_speed: 0.0,
                y_speed: 0.0,
            }
        })
        .collect();

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut instant = Instant::now();

    'main: loop {
        let now = Instant::now();
        let delta = now.duration_since(instant);
        instant = now;

        for event in event_pump.poll_iter() {
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

        gravity_non_linear(&mut droplets, &delta);

        color_buffer.clear(&gl);

        let resolution = na::Vector2::<f32>::new(viewport.w as f32, viewport.h as f32);

        background.render(&gl, &view, &projection.into_inner(), &resolution);

        for droplet_data in &droplets {
            let translation = na::Vector3::new(droplet_data.x, droplet_data.y, 5.0);

            let model = na::Matrix4::<f32>::new_translation(&translation)
                * na::Matrix4::<f32>::new_scaling(droplet_data.size);

            droplet.render(
                &gl,
                &model,
                &view,
                &projection.into_inner(),
                &resolution,
                &translation,
            );
        }

        //        let size: f32 = 80.0;
        //        let model = na::Matrix4::<f32>::new_translation(&na::Vector3::new(
        //            viewport.w as f32 / 2.0 - size,
        //            viewport.h as f32 / 2.0 - size,
        //            5.0,
        //        )) * na::Matrix4::new_scaling(size);
        //
        //        drop.render(&gl, &model, &view, &projection.into_inner(), &resolution);

        window.gl_swap_window();
    }

    Ok(())
}

const PRIVATE_GRAVITY_FORCE_FACTOR_Y: f32 = 0.2;
const PRIVATE_GRAVITY_FORCE_FACTOR_X: f32 = 0.0;

fn gravity_non_linear(droplets: &mut Vec<Droplet>, dt: &Duration) {
    let mut rng = rand::thread_rng();

    let gravity_y = PRIVATE_GRAVITY_FORCE_FACTOR_Y * dt.as_secs_f32();

    for droplet in droplets {
        if droplet.collided {
            droplet.collided = false;
            droplet.seed = (droplet.size * rng.gen::<f32>() * 100.0).floor() as i32;
            droplet.skipping = false;
            droplet.slowing = false;
        } else if droplet.seed <= 0 {
            droplet.seed = (droplet.size * rng.gen::<f32>() * 100.0).floor() as i32;
            droplet.skipping = droplet.skipping == false;
            droplet.slowing = true;
        }

        droplet.seed -= 1;

        if droplet.y_speed > 0.0 {
            if droplet.slowing {
                droplet.y_speed *= 0.9;
                droplet.x_speed *= 0.9;
                if droplet.y_speed < gravity_y {
                    droplet.slowing = false;
                }
            } else if droplet.skipping {
                droplet.y_speed = gravity_y;
                droplet.x_speed = PRIVATE_GRAVITY_FORCE_FACTOR_X;
            } else {
                droplet.y_speed += gravity_y * droplet.size;
                droplet.x_speed += PRIVATE_GRAVITY_FORCE_FACTOR_X * droplet.size;
            }
        } else {
            droplet.y_speed = gravity_y;
            droplet.x_speed = PRIVATE_GRAVITY_FORCE_FACTOR_X;
        }

        //        if this.options.gravityAngleVariance != 0 {
        //            droplet.x_speed +=
        //                (rnd.gen() * 2 - 1) * droplet.y_speed * this.options.gravityAngleVariance
        //        }

        droplet.y -= droplet.y_speed;
        droplet.x += droplet.x_speed;
    }
}
