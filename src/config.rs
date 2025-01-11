use std::{error::Error, fs::File, io::BufReader};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SerialPortConfig {
    pub path: String,
    pub baudrate: u32,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub rawdata: String,
    pub sqlite3: String,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub log4rs_cfg: String,
    pub serial_port: SerialPortConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    pub global: GlobalConfig,
}

impl SystemConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: SystemConfig = serde_json::from_reader(reader)?;

        Ok(config)
    }
}

