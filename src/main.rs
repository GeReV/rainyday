//#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
// mod config_window;
mod debug;
#[cfg(feature = "debug")]
mod debug_ui;
mod droplet;
mod droplets;
mod quad;
mod rain;
pub mod render_gl;
mod vertex;

use crate::config::Config;
// use crate::config_window::ConfigWindow;
use crate::debug::failure_to_string;
#[cfg(feature = "debug")]
use crate::debug_ui::DebugUi;

use glutin::dpi::PhysicalPosition;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
// use glutin::platform::windows::WindowBuilderExtWindows;

use glutin::window::{Fullscreen, WindowBuilder};
use glutin::{ContextBuilder, GlRequest};
use std::env;
use std::str::FromStr;
use std::time::{Duration, Instant};
use glutin::Api::OpenGl;
// use winapi::shared::windef::HWND;
// use winapi::um::winuser::SPI_SCREENSAVERRUNNING;

const MAX_DROPLET_COUNT: usize = 10_000;

enum Mode {
    // Preview(HWND),
    Normal,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    let arg = args
        .get(0)
        .unwrap_or(&"/s".to_string())
        .to_ascii_lowercase();

    match &arg[..2] {
        // "/p" => {
        //     // Preview, parse hwnd from second argument
        //     let hwnd = {
        //         let str_hwnd = args.get(1).unwrap();
        //
        //         usize::from_str(str_hwnd).unwrap()
        //     };
        //
        //     let parent_hwnd = unsafe { std::mem::transmute(hwnd) };
        //
        //     if let Err(e) = run(Mode::Preview(parent_hwnd), 500, (1.0, 5.0)) {
        //         let err = failure_to_string(e);
        //         println!("{}", err);
        //     }
        // }
        // "/c" => {
        //     let (_, _hwnd) = arg.split_at(3);
        //
        //     // Configuration
        //     ConfigWindow::init();
        // }
        "/s" | _ => {
            if let Err(e) = run(Mode::Normal, MAX_DROPLET_COUNT, (3.0, 8.0)) {
                println!("{}", failure_to_string(e));
            }

            std::thread::sleep(Duration::from_secs(5));
        }
    }
}

fn run(
    mode: Mode,
    max_droplet_count: usize,
    droplet_size_range: (f32, f32),
) -> Result<(), failure::Error> {
    let event_loop = EventLoop::new();

    let mut wb = WindowBuilder::new().with_title("Rain");

    wb = match mode {
        // Mode::Preview(hwnd) => {
        //     let mut rect = winapi::shared::windef::RECT {
        //         left: 0,
        //         top: 0,
        //         right: 0,
        //         bottom: 0,
        //     };
        //
        //     unsafe {
        //         winapi::um::winuser::GetClientRect(hwnd, &mut rect);
        //     }
        //
        //     wb.with_parent_window(hwnd)
        //         .with_decorations(false)
        //         .with_inner_size(Size::Physical(PhysicalSize::new(
        //             rect.right as u32,
        //             rect.bottom as u32,
        //         )))
        // }
        Mode::Normal => wb
            .with_visible(false)
            .with_fullscreen(Some(Fullscreen::Borderless(event_loop.primary_monitor()))),
    };

    let raw_context = {
        // use glutin::platform::windows::{RawContextExt, WindowExtWindows};

        // let hwnd = window.hwnd();
        let cb = ContextBuilder::new().with_gl(GlRequest::Specific(OpenGl, (4, 1)));

        cb.build_windowed(wb, &event_loop).unwrap()
    };

    let raw_context = unsafe { raw_context.make_current().unwrap() };

    let gl = gl::Gl::load_with(|s| raw_context.get_proc_address(s) as *const _);

    // if let Mode::Normal = mode {
    //     window.set_visible(true);
    //     window.set_cursor_visible(false);
    //
    //     // set_screensaver_running(true);
    // }

    // let window_size = window.inner_size();
    let window_size = raw_context.window().inner_size();

    #[cfg(feature = "debug")]
    let mut debug_ui = DebugUi::new(&window, &raw_context);

    unsafe {
        gl.Enable(gl::BLEND);
        gl.BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    }

    // let mut context = Option::from(raw_context);

    let mut rain = rain::Rain::new(
        &gl,
        max_droplet_count,
        droplet_size_range,
        (window_size.width, window_size.height),
        &Config::default(),
    )?;

    let mut instant = Instant::now();
    let mut delta = Duration::default();

    let mut initial_mouse_position: Option<PhysicalPosition<f64>> = None;
    let mut skipped_initial_keyboard_events = false;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        #[cfg(feature = "debug")]
        debug_ui.handle_event(&window, &event);

        match event {
            Event::NewEvents(_) => {
                let now = Instant::now();

                delta = now.duration_since(instant);

                #[cfg(feature = "debug")]
                debug_ui.update(&delta);

                instant = now;
            }
            Event::MainEventsCleared => {
                // For some reason, the first round of the loop comes with some initial keyboard events.
                //  This flag is used to ignore them, until I can figure out why they're there in the first place.
                skipped_initial_keyboard_events = true;

                rain.update(&delta);

                raw_context.window().request_redraw();
            }
            Event::LoopDestroyed => {
                // raw_context.take(); // Make sure it drops first
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CursorMoved { position, .. } => match initial_mouse_position {
                    Some(p) => {
                        if (position.x - p.x).abs() > 50.0 || (position.y - p.y).abs() > 50.0 {
                            *control_flow = ControlFlow::Exit;
                        }
                    }
                    None => {
                        initial_mouse_position = Some(position);
                    }
                },
                WindowEvent::KeyboardInput { .. } => {
                    if skipped_initial_keyboard_events {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                WindowEvent::MouseWheel { .. }
                | WindowEvent::MouseInput { .. }
                | WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;

                    // if let Mode::Normal = mode {
                    //     set_screensaver_running(false);
                    // }
                }
                _ => (),
            },
            Event::RedrawRequested(_) => {
                rain.render(&delta);

                #[cfg(feature = "debug")]
                debug_ui.render(
                    &window,
                    rain.droplets.used_count(),
                    rain.droplets_accumulator,
                );

                raw_context.swap_buffers().unwrap();
            }
            _ => (),
        }
    });
}

// fn set_screensaver_running(value: bool) {
//     unsafe {
//         winapi::um::winuser::SystemParametersInfoA(
//             SPI_SCREENSAVERRUNNING,
//             u32::from(value),
//             std::ptr::null_mut(),
//             0,
//         );
//     }
// }
