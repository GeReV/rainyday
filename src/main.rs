extern crate gl;
extern crate sdl2;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate render_gl_derive;
extern crate nalgebra;
extern crate vec_2_10_10_10;

mod background;
mod debug;
mod quad;
pub mod render_gl;
pub mod resources;

use crate::debug::failure_to_string;
use failure::err_msg;
use nalgebra as na;
use render_gl::buffer;
use render_gl::data;
use resources::Resources;
use sdl2::init;
use std::ffi::{CStr, CString};
use std::path::Path;

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

    let quad = quad::Quad::new(&res, &gl)?;

    let background = background::Background::new(
        &res,
        &gl,
        initial_window_size.0 as u32,
        initial_window_size.1 as u32,
    )?;

    viewport.set_used(&gl);

    let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.5));

    color_buffer.set_used(&gl);

    let mut event_pump = sdl.event_pump().map_err(err_msg)?;

    'main: loop {
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
                    projection.set_bottom_and_top(h as f32, 0.0);
                }
                _ => {}
            }
        }

        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
        }

        color_buffer.clear(&gl);

        background.render(
            &gl,
            &view,
            &projection.into_inner(),
            &na::Vector2::<f32>::new(viewport.w as f32, viewport.h as f32),
        );

        window.gl_swap_window();
    }

    Ok(())
}
