pub mod captcha;
pub mod extitem;
pub mod item;
pub mod search;

use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Clone, Deserialize, Serialize)]
pub struct Config {
    pub cookie: String,
    pub base_url: String,
}

impl Config {
    pub fn dump_config(&self) {
        let json = serde_json::to_string(self).unwrap();

        let home = std::env::var("HOME").unwrap();
        let path = format!("{}/.config/rarbg", home);

        std::fs::create_dir_all(path).unwrap();

        File::create(format!("{}/.config/rarbg/config", home))
            .and_then(|mut x| x.write(json.as_bytes()))
            .unwrap();
    }
}
