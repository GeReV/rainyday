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
use nalgebra as na;
use nalgebra::{Isometry2, Vector2};
use ncollide2d::pipeline::{
    CollisionGroups, CollisionObjectSlabHandle, CollisionWorld, GeometricQueryType,
};
use ncollide2d::query::Proximity;
use ncollide2d::shape::{Ball, ShapeHandle};

use crate::render_gl::ColorBuffer;
use glutin::dpi::{PhysicalSize, Size};
use glutin::event_loop::EventLoop;
use glutin::window::Fullscreen;
use glutin::{Context, ContextWrapper, GlRequest, PossiblyCurrent};
use render_gl::buffer::*;
use resources::Resources;
use std::env;
use std::path::Path;
use std::rc::Rc;
use std::str::FromStr;
use std::time::{Duration, Instant};
use winapi::shared::windef::HWND;

const MAX_DROPLET_COUNT: usize = 10_000;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let arg = args
        .get(0)
        .unwrap_or(&"/s".to_string())
        .to_ascii_lowercase();

    match &arg[..2] {
        "/p" => {
            // Preview, parse hwnd from second argument
            let hwnd = {
                let str_hwnd = args.get(1).unwrap();

                usize::from_str(str_hwnd).unwrap()
            };

            let parent_hwnd = unsafe { std::mem::transmute(hwnd) };

            if let Err(e) = run(Some(parent_hwnd), 500, (1.0, 5.0)) {
                let err = failure_to_string(e);
                println!("{}", err);
            }
        }
        "/c" => {
            let (_, hwnd) = arg.split_at(3);

            // Configuration
            ConfigWindow::init();
        }
        "/s" | _ => {
            if let Err(e) = run(None, MAX_DROPLET_COUNT, (3.0, 8.0)) {
                println!("{}", failure_to_string(e));
            }
        }
    }
}

fn run(
    parent_hwnd: Option<HWND>,
    max_droplet_count: usize,
    droplet_size_range: (f32, f32),
) -> Result<(), failure::Error> {
    use glutin::event::{Event, WindowEvent};
    use glutin::event_loop::{ControlFlow, EventLoop};
    use glutin::platform::windows::{RawContextExt, WindowBuilderExtWindows, WindowExtWindows};
    use glutin::window::WindowBuilder;
    use glutin::ContextBuilder;

    let event_loop = EventLoop::new();

    let mut wb = WindowBuilder::new().with_title("Rain");

    wb = match parent_hwnd {
        Some(hwnd) => {
            let mut rect = winapi::shared::windef::RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };

            unsafe {
                winapi::um::winuser::GetClientRect(hwnd, &mut rect);
            }

            wb.with_parent_window(hwnd)
                .with_decorations(false)
                .with_inner_size(Size::Physical(PhysicalSize::new(
                    rect.right as u32,
                    rect.bottom as u32,
                )))
        }
        None => wb.with_fullscreen(Some(Fullscreen::Borderless(None))),
    };

    let window = wb.build(&event_loop).unwrap();

    let raw_context = unsafe {
        use glutin::platform::windows::{RawContextExt, WindowExtWindows};

        let hwnd = window.hwnd();
        let mut cb = ContextBuilder::new().with_gl(GlRequest::Latest);

        cb.build_raw_context(hwnd).unwrap()
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

    let mut context = Option::from(raw_context);

    let mut rain = rain::Rain::new(
        &gl,
        max_droplet_count,
        droplet_size_range,
        (window_size.width, window_size.height),
        &Config::default(),
    )?;

    let mut instant = Instant::now();
    let mut delta = Duration::default();

    event_loop.run(move |event, _, control_flow| {
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

                window.request_redraw();
            }
            Event::LoopDestroyed => {
                context.take(); // Make sure it drops first
                return;
            }
            Event::WindowEvent { event, .. } => match event {
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

                context.as_ref().unwrap().swap_buffers().unwrap();
            }
            _ => (),
        }
    });

    Ok(())
}
