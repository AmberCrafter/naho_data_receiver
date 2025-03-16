use std::{
    collections::HashMap,
    error::Error,
    fs::rename,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
};

use chrono::NaiveDateTime;
use codec::{CodecConfigDB, CodecConfigMetadata};

use crate::config::placeholder_get_tag;

pub mod codec;
pub mod parser_cwb;
pub mod qc_level1;
pub mod reader_loggernet;
pub mod reader_serial_port;
pub mod receiver_raw;
pub mod receiver_sqlite;

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

#[derive(Debug)]
pub struct HeaderTableValue {
    hash: u64,
    header: Vec<String>,
    is_update: bool,
}
pub type HeaderTable = HashMap<String, HeaderTableValue>;

fn cal_hash<T>(t: T) -> u64
where
    T: Hash,
{
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn is_update_header(table: &mut HeaderTable, key: &str, header: &Vec<String>) -> bool {
    let header_hash = cal_hash(&header);
    let key = key.to_string();

    // check previous header hash
    if let Some(val) = table.get(&key) {
        if val.hash == header_hash {
            log::info!("header hash is same.");
            return false;
        }
    }

    // update table
    table
        .entry(key.clone())
        .and_modify(|v| {
            v.hash = header_hash;
            v.header = header.clone();
            v.is_update = true;
        })
        .or_insert(HeaderTableValue {
            hash: header_hash,
            header: header.clone(),
            is_update: true,
        });
    true
}

pub fn backup_file<P>(filepath: P) -> Result<(), Box<dyn Error + 'static>>
where
    P: AsRef<Path>,
{
    let filepath = filepath.as_ref().to_owned();
    let directory = filepath.parent().unwrap();
    let mut id = 1;
    while id <= 100 {
        let newname = format!(
            "{}_{}.{}",
            filepath.file_stem().unwrap().to_str().unwrap(),
            id,
            filepath.extension().unwrap().to_str().unwrap()
        );

        let new_filepath = Path::new(directory).join(newname);
        if !new_filepath.exists() {
            match rename(&filepath, &new_filepath) {
                Ok(_) => {
                    log::info!("Rename {:?} to {:?}", filepath, new_filepath);
                }
                Err(e) => {
                    log::error!("{}: Rename {:?} to {:?} failed.", e, filepath, new_filepath);
                    return Err(Box::new(e));
                }
            }
            break;
        }

        id += 1;

        if id >= 100 {
            log::error!("Backup index overflow.");
            return Err(String::from("Backup index overflow!").into());
        }
    }
    Ok(())
}

pub fn generate_db_filepath(
    tag: &str,
    db_config: &CodecConfigDB,
    data_config: &CodecConfigMetadata,
    opts: &HashMap<String, String>,
) -> Option<PathBuf> {
    fn gen_default_filename(tag: &str, opts: &HashMap<String, String>) -> String {
        if let Some(datetime) = opts.get("datetime") {
            if let Ok(datetime) = NaiveDateTime::parse_from_str(&datetime, DTAETIME_FMT) {
                return format!("{}_{}", tag, datetime.format("%Y%m%d"));
            } else {
                log::error!("System Error: {:?}", datetime);
            }
        } else {
            log::warn!("Unsupport datetime");
        }
        format!("{}", tag)
    }

    let mut filepath = PathBuf::from(&db_config.directory);

    if let Some(seperate_by) = &db_config.seperate_by {
        if let Some(val) = placeholder_get_tag(seperate_by.as_str()) {
            match val {
                val if val.starts_with("metadatas.") => {
                    filepath.push(data_config.name.as_str());
                }
                _ => {}
            }
        } else {
            log::error!("Invalid: {:?}", seperate_by);
        }
    }

    let suffix = if let Some(suffix) = &db_config.suffix {
        suffix.as_str()
    } else {
        "dat"
    };

    if let Some(pattern) = &db_config.pattern {
        if let Ok(filename) = data_config.replace_placeholder(&pattern, opts) {
            filepath.push(filename);
        } else {
            filepath.push(format!("{}.{}", gen_default_filename(tag, opts), suffix));
        }
    } else {
        filepath.push(format!("{}.{}", gen_default_filename(tag, opts), suffix));
    }
    Some(filepath)
}
