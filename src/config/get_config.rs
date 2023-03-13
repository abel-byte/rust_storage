use std::{fs::File, io::Read};

use once_cell::sync::Lazy;

use super::configs::Configs;

const CONFIG_FILE: &str = "config/default.toml";
pub static CONFIG: Lazy<Configs> = Lazy::new(self::Configs::init);

impl Configs {
    pub fn init() -> Self {
        let mut file = match File::open(CONFIG_FILE) {
            Ok(f) => f,
            Err(err) => panic!("config file {} not exists, error: {}", CONFIG_FILE, err),
        };
        let mut contents = String::new();
        match file.read_to_string(&mut contents) {
            Ok(size) => size,
            Err(err) => panic!("read config file failed, err: {}", err),
        };
        toml::from_str(&contents).expect("parse config file failed")
    }
}
