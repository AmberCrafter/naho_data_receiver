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
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub log4rs_cfg: String,
    pub serial_port: SerialPortConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Deserialize)]
pub struct DataTypeConfig {
    pub rust: String,
    pub sqlite: String,
}

#[derive(Debug, Deserialize)]
pub struct DataConfig {
    pub name: String,
    pub sqlite_name: String,
    pub datatype: DataTypeConfig,
    pub regexp: String,
}

#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    pub global: GlobalConfig,
    pub data: Vec<DataConfig>,
}

impl SystemConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: SystemConfig = serde_json::from_reader(reader)?;

        Ok(config)
    }
}

