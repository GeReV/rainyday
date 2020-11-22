use imgui::Ui;
use sdl2::event::Event;
use sdl2::mouse::MouseState;
use sdl2::video::Window;
use std::collections::VecDeque;
use std::time::Duration;

pub struct DebugUi {
    imgui_context: imgui::Context,
    imgui_sdl2: imgui_sdl2::ImguiSdl2,
    renderer: imgui_opengl_renderer::Renderer,

    opened: bool,
    frames: VecDeque<f32>,
}

impl DebugUi {
    pub fn new(window: &Window) -> Self {
        let mut imgui_context = imgui::Context::create();
        imgui_context.set_ini_filename(None);

        let imgui_sdl2 = imgui_sdl2::ImguiSdl2::new(&mut imgui_context, &window);

        let renderer = imgui_opengl_renderer::Renderer::new(&mut imgui_context, |s| {
            window.subsystem().gl_get_proc_address(s) as _
        });

        DebugUi {
            imgui_context,
            imgui_sdl2,
            renderer,
            opened: true,
            frames: VecDeque::with_capacity(100),
        }
    }

    pub fn handle_event(&mut self, event: &Event) {
        self.imgui_sdl2
            .handle_event(&mut self.imgui_context, &event);
    }

    pub fn ignore_event(&self, event: &Event) -> bool {
        self.imgui_sdl2.ignore_event(&event)
    }

    pub fn render(
        &mut self,
        window: &Window,
        mouse_state: &MouseState,
        delta: &Duration,
        droplets_used_count: usize,
        droplets_accumulator: usize,
    ) {
        self.imgui_sdl2
            .prepare_frame(self.imgui_context.io_mut(), &window, mouse_state);

        self.imgui_context.io_mut().delta_time = delta.as_secs_f32();

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

        self.imgui_sdl2.prepare_render(&ui, &window);

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
