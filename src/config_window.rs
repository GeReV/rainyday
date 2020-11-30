extern crate native_windows_derive as nwd;
extern crate native_windows_gui as nwg;

use crate::config::Config;
use nwd::NwgUi;
use nwg::stretch::{
    geometry::Size,
    style::{AlignItems, Dimension as D, FlexDirection},
};
use nwg::NativeUi;
use std::cell::RefCell;
use std::env;
use std::path::{Path, PathBuf};
use winapi::um::winuser::{SM_CXSCREEN, SM_CYSCREEN};

#[derive(Default, NwgUi)]
pub struct ConfigWindow {
    // The image that will be loaded dynamically
    loaded_image: RefCell<Option<nwg::Bitmap>>,

    #[nwg_control(size: (520, 160), position: (400, 150), title: "Rainy Day Screensaver")]
    #[nwg_events(OnInit: [ConfigWindow::open], OnWindowClose: [ConfigWindow::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, flex_direction: FlexDirection::Row, align_items: AlignItems::Center )]
    main_layout: nwg::FlexboxLayout,

    #[nwg_resource]
    decoder: nwg::ImageDecoder,

    #[nwg_resource(title: "Open File", action: nwg::FileDialogAction::Open, filters: "Images (*.png;*.jpg;*.jpeg;*.dds;*.tiff;*.bmp)|Any (*.*)")]
    dialog: nwg::FileDialog,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: main_layout, size: Size { width: D::Percent(1.0), height: D::Points(30.0) })]
    file_name: nwg::TextInput,

    #[nwg_control(text: "Browse", focus: true)]
    #[nwg_layout_item(layout: main_layout, min_size: Size { width: D::Points(100.0), height: D::Points(32.0) })]
    #[nwg_events(OnButtonClick: [ConfigWindow::open_file])]
    open_btn: nwg::Button,

    #[nwg_control]
    #[nwg_layout_item(layout: main_layout, min_size: Size { width: D::Points(200.0), height: D::Points(150.0) })]
    img: nwg::ImageFrame,
}

impl ConfigWindow {
    pub fn init() {
        nwg::init().expect("Failed to init Native Windows GUI");
        nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

        let _app = ConfigWindow::build_ui(Default::default()).expect("Failed to build UI");

        nwg::dispatch_thread_events();
    }

    fn open_file(&self) {
        if let Ok(d) = env::current_dir() {
            if let Some(d) = d.to_str() {
                self.dialog
                    .set_default_folder(d)
                    .expect("Failed to set default folder.");
            }
        }

        if self.dialog.run(Some(&self.window)) {
            self.file_name.set_text("");
            if let Ok(directory) = self.dialog.get_selected_item() {
                self.file_name.set_text(&directory);
                self.read_file();

                if let Err(err) = self.save(PathBuf::from(self.file_name.text()).as_path()) {
                    println!("Could not save path in registry: {:?}", err);
                };
            }
        }
    }

    fn read_file(&self) {
        let image = match self.decoder.from_filename(&self.file_name.text()) {
            Ok(img) => img,
            Err(_) => {
                println!("Could not read image!");
                return;
            }
        };

        let frame = match image.frame(0) {
            Ok(bmp) => bmp,
            Err(_) => {
                println!("Could not read image frame!");
                return;
            }
        };

        let size = frame.size();
        let resize_factor = self.img.size().0 as f32 / size.0 as f32;
        let thumbnail = {
            let t = self.decoder.resize_image(
                &frame,
                [
                    (size.0 as f32 * resize_factor) as u32,
                    (size.1 as f32 * resize_factor) as u32,
                ],
            );

            match t {
                Ok(bmp) => bmp,
                Err(_) => {
                    println!("Could not resize image!");
                    return;
                }
            }
        };

        // Create a new Bitmap image from the image data
        match thumbnail.as_bitmap() {
            Ok(bitmap) => {
                let mut img = self.loaded_image.borrow_mut();
                img.replace(bitmap);
                self.img.set_bitmap(img.as_ref());
            }
            Err(_) => {
                println!("Could not convert image to bitmap!");
            }
        }
    }

    fn save(&self, path: &Path) -> std::io::Result<()> {
        use winapi::um::winuser::GetSystemMetrics;

        let config = Config::default();

        // Remove previous cache.
        if let Some(previous_background) = config.cached_background() {
            if let Err(e) = std::fs::remove_file(&previous_background) {
                eprintln!(
                    "Failed to delete file {}: {}",
                    previous_background.to_string_lossy(),
                    e.to_string()
                )
            }
        }

        let _ = config.set_background(path);

        let image = image::open(path).unwrap();

        let width = unsafe { GetSystemMetrics(SM_CXSCREEN) };
        let height = unsafe { GetSystemMetrics(SM_CYSCREEN) };

        let resized_image =
            image.resize_to_fill(width as u32, height as u32, image::FilterType::Gaussian);

        resized_image.save(config.cached_background().unwrap().as_path())
    }

    fn open(&self) {
        if let Some(path) = Config::default().background() {
            self.file_name.set_text(path.to_str().unwrap());
            self.read_file();
        }
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}
