#![windows_subsystem = "windows"]

extern crate gl;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate render_gl_derive;
extern crate nalgebra;
extern crate ncollide2d;
extern crate rand;

mod background;
mod config;
mod config_window;
mod debug;
#[cfg(feature = "debug")]
mod debug_ui;
mod droplet;
mod droplets;
mod quad;
mod rain;
pub mod render_gl;
pub mod resources;
mod vertex;

use crate::config::Config;
use crate::config_window::ConfigWindow;
use crate::debug::failure_to_string;
#[cfg(feature = "debug")]
use crate::debug_ui::DebugUi;
use crate::droplets::Droplets;
use crate::quad::Quad;

use failure::err_msg;
use image::GenericImageView;
use nalgebra as na;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::pipeline::{
    CollisionGroups, CollisionObjectSlabHandle, CollisionWorld, GeometricQueryType,
};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, ShapeHandle};

use render_gl::buffer::*;
use resources::Resources;
use std::env;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

const MAX_DROPLET_COUNT: usize = 10_000;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let arg = args.get(0);

    match &arg.unwrap_or(&"/s".to_string()).to_ascii_lowercase()[..2] {
        "/p" => {
            // Preview, parse hwnd from second argument
        }
        "/c" => {
            // Configuration
            ConfigWindow::init();
        }
        "/s" | _ => {
            if let Err(e) = run() {
                println!("{}", failure_to_string(e));
            }
        }
    }
}

fn run() -> Result<(), failure::Error> {
    use glutin::event::{Event, WindowEvent};
    use glutin::event_loop::{ControlFlow, EventLoop};
    use glutin::window::WindowBuilder;
    use glutin::ContextBuilder;

    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

    let el = EventLoop::new();
    let mut wb = WindowBuilder::new()
        .with_title("Rain")
        .with_inner_size(Size::Physical(PhysicalSize::new(1920, 1080)));

    let window = wb.build(&el)?;

    let raw_context = unsafe {
        use glutin::platform::windows::{RawContextExt, WindowExtWindows};

        let hwnd = window.hwnd();
        ContextBuilder::new().build_raw_context(hwnd)?
    };

    let raw_context = unsafe { raw_context.make_current().unwrap() };

    let gl = gl::Gl::load_with(|s| raw_context.get_proc_address(s) as *const _);

    let window_size = window.inner_size();

    // #[cfg(feature = "debug")]
    // let mut debug_ui = DebugUi::new(&window);

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    let mut raw_context = Option::from(raw_context);

    let mut rain = rain::Rain::new(
        &gl,
        MAX_DROPLET_COUNT,
        (window_size.width, window_size.height),
        &Config::default(),
    )?;

    let mut instant = Instant::now();
    let mut delta = Duration::default();

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // #[cfg(feature = "debug")]
        //     {
        //         debug_ui.handle_event(&event);
        //
        //         if debug_ui.ignore_event(&event) {
        //             continue;
        //         }
        //     }

        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();

                delta = now.duration_since(instant);

                instant = now;
            }
            Event::MainEventsCleared => {
                rain.update(&delta);
            }
            Event::LoopDestroyed => {
                raw_context.take(); // Make sure it drops first
                return;
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    // viewport.update_size(w, h);
                    // viewport.set_used(&gl);
                    //
                    // projection.set_left_and_right(0.0, w as f32);
                    // projection.set_bottom_and_top(0.0, h as f32);
                    //
                    // raw_context.resize(physical_size);
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            Event::RedrawRequested(_) => {
                rain.render(&delta);

                // #[cfg(feature = "debug")]
                // debug_ui.render(
                //     &window,
                //     &event_pump.mouse_state(),
                //     &delta,
                //     droplets.used_count(),
                //     droplets_accumulator,
                // );

                raw_context.as_ref().unwrap().swap_buffers().unwrap();
            }
            _ => (),
        }
    });

    Ok(())
}
