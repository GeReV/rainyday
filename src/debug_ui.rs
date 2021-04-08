use glutin::event::Event;
use glutin::window::Window;
use glutin::{ContextWrapper, PossiblyCurrent};
use imgui::Ui;
use imgui_winit_support::HiDpiMode;
use std::collections::VecDeque;
use std::time::Duration;

pub struct DebugUi {
    imgui_context: imgui::Context,
    platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_opengl_renderer::Renderer,

    opened: bool,
    frames: VecDeque<f32>,
}

impl DebugUi {
    pub fn new<W>(window: &Window, context: &ContextWrapper<PossiblyCurrent, W>) -> Self {
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);

        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);

        platform.attach_window(imgui_context.io_mut(), window, HiDpiMode::Rounded);

        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui_context, |s| {
            context.get_proc_address(s) as _
        });

        DebugUi {
            imgui_context,
            platform,
            renderer,
            opened: true,
            frames: VecDeque::with_capacity(100),
        }
    }

    pub fn handle_event<T>(&mut self, window: &Window, event: &Event<T>) {
        self.platform
            .handle_event(self.imgui_context.io_mut(), window, event);
    }

    pub fn update(&mut self, delta: &Duration) {
        self.imgui_context.io_mut().delta_time = delta.as_secs_f32();
    }

    pub fn render(
        &mut self,
        window: &Window,
        droplets_used_count: usize,
        droplets_accumulator: usize,
    ) {
        self.platform
            .prepare_frame(self.imgui_context.io_mut(), window)
            .unwrap();

        if self.frames.len() == 100 {
            self.frames.pop_front();
        }
        self.frames.push_back(self.imgui_context.io().framerate);

        let mut ui = self.imgui_context.frame();

        Self::build_ui(
            &mut ui,
            &mut self.frames,
            &mut self.opened,
            droplets_used_count,
            droplets_accumulator,
        );

        self.platform.prepare_render(&ui, window);

        self.renderer.render(ui);
    }

    fn build_ui(
        ui: &mut Ui,
        frames: &mut VecDeque<f32>,
        opened: &mut bool,
        droplets_used_count: usize,
        droplets_accumulator: usize,
    ) {
        let w = imgui::Window::new(imgui::im_str!("FPS"))
            .opened(opened)
            .position([20.0, 20.0], imgui::Condition::Appearing)
            .always_auto_resize(true);

        let frames = &*frames;
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
}
