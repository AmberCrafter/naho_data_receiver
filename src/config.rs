use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

use chrono::NaiveDateTime;
use regex::Regex;
use serde::Deserialize;

use crate::component::{
    codec::{CodecConfigBase, CodecConfigMetadata},
    DTAETIME_FMT,
};

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct SerialPortConfig {
    pub path: String,
    pub baudrate: u32,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct ListenConfigFlags {
    pub f_move: Option<bool>,
    pub f_remove_after_used: Option<bool>,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct ListenConfigHeader {
    pub number: usize,
}

#[allow(unused)]
#[derive(Debug, Deserialize, Clone)]
pub struct ListenConfig {
    pub name: String,
    pub path: String,
    pub ftype: String,
    pub tag: String,
    pub dkind: String,
    pub header: Option<ListenConfigHeader>,
    pub flags: Option<ListenConfigFlags>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub log4rs_cfg: String,
    pub serial_port: SerialPortConfig,
    pub listen_move_suffix: Option<String>,
    pub listen_list: Option<Vec<ListenConfig>>,
}

#[allow(unused)]
#[derive(Debug, Deserialize)]
pub struct SystemConfig {
    pub global: GlobalConfig,
    pub codec: HashMap<String, CodecConfigBase>,
}

impl SystemConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn Error + 'static>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let config: SystemConfig = serde_json::from_reader(reader)?;

        Ok(config)
    }
}

pub fn placeholder_get_tag<'a>(placeholder: &'a str) -> Option<&'a str> {
    placeholder.strip_prefix("{{")?.strip_suffix("}}")
}

impl CodecConfigMetadata {
    pub fn replace_placeholder(
        &self,
        raw: &str,
        opts: &HashMap<String, String>,
    ) -> Result<String, Box<dyn Error + 'static>> {
        let re = Regex::new(r"\{\{.*?\}\}")?;
        let mut result = raw.to_string();

        for placeholder in re.find_iter(raw) {
            let Some(val) = placeholder_get_tag(placeholder.as_str()) else {
                log::error!("Invalid: {:?}", placeholder.as_str());
                continue;
            };

            // TODO: need rework
            match val {
                val if val.starts_with("metadatas.") => match val.split(".").nth(1) {
                    Some("name") => {
                        result = result.replace(placeholder.as_str(), &self.name);
                    }
                    _ => {
                        log::error!("Unsupport: {val}");
                    }
                },
                val if val == "DATETIME" => {
                    if let Some(value) = opts.get("datetime") {
                        if let Ok(datetime) = NaiveDateTime::parse_from_str(&value, DTAETIME_FMT) {
                            let dt = datetime.format("%Y%m%d%H%M%S").to_string();
                            result = result.replace(placeholder.as_str(), &dt);
                        } else {
                            log::error!("System Error: {val}");
                        }
                    } else {
                        log::error!("Unsupport: {val}");
                    }
                }
                val if val == "DATE" => {
                    if let Some(value) = opts.get("datetime") {
                        if let Ok(datetime) = NaiveDateTime::parse_from_str(&value, DTAETIME_FMT) {
                            let dt = datetime.format("%Y%m%d").to_string();
                            result = result.replace(placeholder.as_str(), &dt);
                        } else {
                            log::error!("System Error: {val}");
                        }
                    } else {
                        log::error!("Unsupport: {val}");
                    }
                }
                val if val == "TIME" => {
                    if let Some(value) = opts.get("datetime") {
                        if let Ok(datetime) = NaiveDateTime::parse_from_str(&value, DTAETIME_FMT) {
                            let dt = datetime.format("%H%M%S").to_string();
                            result = result.replace(placeholder.as_str(), &dt);
                        } else {
                            log::error!("System Error: {val}");
                        }
                    } else {
                        log::error!("Unsupport: {val}");
                    }
                }
                val => {
                    log::error!("Unsupport: {val}");
                }
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use serde_json::Value;

    use super::*;
    #[test]
    fn general_parse() {
        let path = "./config/config.json";
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let config: Value = serde_json::from_reader(reader).unwrap();

        let codec = config.get("codec").unwrap().as_object().unwrap();

        println!("{:?}", codec.get("cwb").unwrap().get("tag"));
    }

    #[test]
    fn re_placeholder() {
        let re = Regex::new(r"\{\{.*?\}\}").unwrap();
        let raw = "{{datetime}}_{{metadatas.name}}.dat";
        let mut pattern = raw.to_string();

        let tmp = re.find_iter(raw);
        for placeholder in tmp {
            let val = placeholder
                .as_str()
                .strip_prefix("{{")
                .unwrap()
                .strip_suffix("}}")
                .unwrap();
            println!("{}", val);

            match val {
                "datetime" => {
                    pattern = pattern.replace(placeholder.as_str(), "1970-01-01");
                }
                _ => {}
            }
        }
        println!("{:?}", pattern);
    }
}
