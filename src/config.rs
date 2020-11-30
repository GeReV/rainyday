use std::path::{Path, PathBuf};

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

    pub fn background(&self) -> Option<PathBuf> {
        let ini = ini::Ini::load_from_file(&self.path);

        match ini {
            Ok(ini) => ini
                .get_from(None::<&str>, BACKGROUND_KEY)
                .map(|s| PathBuf::from(s)),
            Err(_) => None,
        }
    }

    pub fn cached_background(&self) -> Option<PathBuf> {
        self.background()
            .map(|p| self.backgrounds_directory().join(p.file_name().unwrap()))
    }

    pub fn set_background(&self, filename: &Path) -> std::io::Result<()> {
        let mut ini = ini::Ini::new();

        ini.with_general_section()
            .set(BACKGROUND_KEY, filename.to_str().unwrap());

        ini.write_to_file(&self.path)
    }

    pub fn backgrounds_directory(&self) -> PathBuf {
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("assets\\textures")
    }
}

impl Default for Config {
    fn default() -> Self {
        let path = std::env::current_exe().unwrap().with_extension("ini");

        Config::new(path)
    }
}
