use std::error::Error;

pub mod codec;
pub mod parser_cwb;
pub mod qc_level1;
pub mod reader_loggernet;
pub mod reader_serial_port;
pub mod receiver_raw;
pub mod receiver_sqlite;
pub mod utils;

type INTEGER = i64;
type FLOAT = f64;
pub const DTAETIME_FMT: &str = "%Y-%m-%d %H:%M:%S";

#[derive(Debug, Clone)]
pub struct MsgPayload {
    pub tag: String,
    pub dkind: String,
    pub update_header: bool, // if true, value is header informatino
    pub value: Vec<String>,
}

impl MsgPayload {
    pub fn new(tag: &str, dkind: &str, value: Vec<String>) -> Self {
        MsgPayload {
            tag: tag.to_string(),
            dkind: dkind.to_string(),
            update_header: false,
            value,
        }
    }

    pub fn set_update_header(&mut self) -> Result<(), Box<dyn Error + 'static>> {
        self.update_header = true;
        Ok(())
    }
}
