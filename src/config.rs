use ini::{Error, Ini};
use std::path::Path;

const BACKGROUND_KEY: &str = "background";

pub struct Config {
    path: String,
}

impl Config {
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Config {
            path: path
                .as_ref()
                .to_str()
                .expect("Unable to convert path to str.")
                .to_string(),
        }
    }

    pub fn background(&self) -> Option<String> {
        let ini = ini::Ini::load_from_file(&self.path);

        match ini {
            Ok(ini) => ini
                .get_from(None::<&str>, BACKGROUND_KEY)
                .map(|s| s.to_string()),
            Err(_) => None,
        }
    }

    pub fn set_background(&self, filename: &str) -> std::io::Result<()> {
        let mut ini = ini::Ini::new();

        ini.with_general_section().set(BACKGROUND_KEY, filename);

        ini.write_to_file(&self.path)
    }
}

impl Default for Config {
    fn default() -> Self {
        let path = std::env::current_exe().unwrap().with_extension("ini");

        Config::new(path)
    }
}
