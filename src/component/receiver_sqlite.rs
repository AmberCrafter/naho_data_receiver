use std::{
    collections::HashSet, error::Error, fmt::Display, fs::{create_dir_all, rename}, path::Path, sync::mpsc::Receiver, thread::{self, JoinHandle}
};

use chrono::{NaiveDateTime, NaiveTime};
use regex::Regex;
use sqlite::Connection;
use crate::component::parser_cwb::CWBCodecConfig;
use super::parser_cwb::CWBDataConfig;

#[derive(Debug)]
enum SQLiteErrorType {
    Unknown,
    Invalid,
    NotExist,
    NotMatch,
}

impl Display for SQLiteErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SQLiteErrorType::Unknown => write!(f, "SQLiteErrorType::Unknown"),
            SQLiteErrorType::Invalid => write!(f, "SQLiteErrorType::Invalid"),
            SQLiteErrorType::NotExist => write!(f, "SQLiteErrorType::NotExist"),
            SQLiteErrorType::NotMatch => write!(f, "SQLiteErrorType::NotMatch"),
        }
    }
}

impl Error for SQLiteErrorType {}

fn get_last_modified_file<P>(root: P, re: Regex) -> Option<String>
where
    P: AsRef<Path>,
{
    let Ok(dir) = std::fs::read_dir(&root) else {log::error!("Couldn't access local directory"); return None};
    let Some(last_modified_file) = dir    
        .flatten() // Remove failed
        .filter(|f| f.metadata().unwrap().is_file()) // Filter out directories (only consider files)
        .filter(|f| re.is_match(f.file_name().to_str().unwrap()) ) // Filter out directories (only consider files)
        .max_by_key(|x| x.metadata().unwrap().modified().unwrap()) // Get the most recently modified file
        else {return None};

    if let Some(filename) = last_modified_file.file_name().to_str() {
        Some(filename.to_string())
    } else {
        None
    }
}

fn create_db<P>(path: P, config: &CWBCodecConfig) -> Result<(), Box<dyn Error + 'static>>
where
    P: AsRef<Path>,
{
    let connection = sqlite::open(path)?;
    for mem in config.codec.iter() {
        for dkind in mem.dkind.iter() {
            if let Some(statement) = config.gen_sqlite3_create_table_cmd(dkind, &mem.name) {
                connection.execute(statement)?;
            }
        }
    }

    Ok(())
}

fn check_column<P>(path: P, config: &CWBCodecConfig) -> Result<(), SQLiteErrorType>
where
    P: AsRef<Path>,
{
    let Ok(connection) = sqlite::open(path) else {return Err(SQLiteErrorType::Invalid)};
    for dconfig in config.codec.iter() {
        let mut set = HashSet::new();
        let query = format!("PRAGMA table_info({});", dconfig.name);
        let Ok(mut statement) = connection.prepare(query) else {return Err(SQLiteErrorType::Invalid)};
        while let Ok(sqlite::State::Row) = statement.next() {
            if let Ok(name) = statement.read::<String, _>("name") {
                set.insert(name);
            }
        }

        for cfg_name in &dconfig.formation {
            if !set.contains(&cfg_name.sqlite3.name) {return Err(SQLiteErrorType::NotMatch);}
        }
    }
    Ok(())
}

pub fn parse_rawdata(rawdata: &str, config: &CWBDataConfig) -> Option<String> {
    let mut buf = Vec::new();
    let mut words = rawdata.split(',');

    // stx
    if words.next() != Some("\u{2}") {
        return None;
    }

    for dtype in config.formation.iter() {
        let Some(subdata) = words.next() else {
            log::error!("System error!");
            return None;
        };

        match (dtype.sqlite3.dtype.as_str(), dtype.sqlite3.unit.as_deref()) {
            ("TEXT", Some("YY-mm-dd HH:MM:SS")) => {
                let Ok(temp) = NaiveDateTime::parse_from_str(subdata, "%Y%m%d%H%M") else {
                    log::error!("Invalid data!");
                    return None;
                };
                buf.push(temp.format("'%Y-%m-%d %H:%M:%S'").to_string());
            }
            ("TEXT", Some("HH:MM:SS")) => {
                let Ok(temp) = NaiveTime::parse_from_str(subdata, "%H%M") else {
                    log::error!("Invalid data!");
                    return None;
                };
                buf.push(temp.format("'%H:%M:%S'").to_string());
            }
            ("TEXT", _) => buf.push(format!("'{}'", subdata)),
            _ => buf.push(subdata.to_string()),
        }
    }

    // etx
    if words.next() != Some("\u{3}") {
        return None;
    }
    Some(buf.join(","))
}

pub fn setup_sqlite3_recorder(
    receiver: Receiver<String>,
    root: &str,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    create_dir_all(root)?;
    let root = root.to_string();

    let handler = thread::spawn(move || {
        let cwb_codec_config =
            CWBCodecConfig::load("config/config.json").expect("load config failed");
        log::info!(target: "configuation", "{cwb_codec_config:?}");

        if let Ok(re) = Regex::new(r"CWB_[0-9]{8}.sql") {
            if let Some(last_db) = get_last_modified_file(&root, re) {
                let filepath = Path::new(&root).join(last_db);
                match check_column(&filepath, &cwb_codec_config) {
                    Err(SQLiteErrorType::NotMatch) => {
                        log::info!("check_column: {:?}", SQLiteErrorType::NotMatch);
                        let mut id = 1;
                        while id <= 100 {
                            let newname = format!("{}_{}.{}",
                                filepath.file_stem().unwrap().to_str().unwrap(),
                                id,
                                filepath.extension().unwrap().to_str().unwrap()
                            );

                            let new_filepath = Path::new(&root).join(newname);
                            if !new_filepath.exists() {
                                match rename(&filepath, &new_filepath) {
                                    Ok(_) => {log::info!("Rename {:?} to {:?}", filepath, new_filepath);}
                                    Err(e) => {log::error!("{}: Rename {:?} to {:?} failed.", e, filepath, new_filepath);}
                                }
                                break;
                            }

                            id+=1;
                        }
                    },
                    Ok(()) => {}
                    ret => {log::error!("{:?}", ret);}
                }
            };
        } else {
            log::error!("System error: create regex failed");
        }

        loop {
            while let Ok(msg) = receiver.recv() {
                let mut words = msg.split(',');

                let Some(dkind) = words.nth(2) else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let Some(dconfig) = cwb_codec_config.get_data_config(dkind) else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let columnname = dconfig
                    .formation
                    .iter()
                    .map(|mem| mem.sqlite3.name.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                let Some(timestr) = words.next() else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let Ok(time) = NaiveDateTime::parse_from_str(timestr, "%Y%m%d%H%M") else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let Some(data_str) = parse_rawdata(&msg, dconfig) else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let filename = format!("CWB_{}.sql", time.format("%Y%m%d"));
                let filepath = Path::new(&root).join(filename);

                if !filepath.exists() {
                    match create_db(&filepath, &cwb_codec_config) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Create database failed: {e} - {filepath:?} - {msg}");
                            continue;
                        }
                    }
                }

                if let Ok(connection) = sqlite::open(filepath) {
                    let statement = if dconfig.raw_save == Some(true) {
                        format!(
                            "INSERT into {} ({},rawdata) values ({},'{}');",
                            &dconfig.name, columnname, data_str, msg.clone()
                        )
                    } else {
                        format!(
                            "INSERT into {} ({}) values ({});",
                            &dconfig.name, columnname, data_str
                        )
                    };
                    match connection.execute(&statement) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Insert data failed: {e} - {statement}");
                            continue;
                        }
                    }
                } else {
                    log::error!("Open database failed: {msg}");
                    continue;
                }
            }
        }
        0
    });
    Ok(handler)
}

#[cfg(test)]
mod test {
    use sqlite::State;

    use super::*;
    #[test]
    fn setup_data() {
        let connection = sqlite::open("data/sqlite3/test.sql").unwrap();

        let table_info = "
            name TEXT,
            age INTEGER
        ";

        let query = format!(
            "
                create table data ({table_info});
                insert into data values ('alice', 10);
                insert into data values ('bob', 20);
            "
        );

        connection.execute(query).unwrap();
    }

    #[test]
    fn get_data1() {
        let connection = sqlite::open("data/sqlite3/test.sql").unwrap();
        let query = format!("select * from data where age > 5");
        connection
            .iterate(query, |datas| {
                for &(name, value) in datas.iter() {
                    println!("{name}: {value:?}");
                }
                true
            })
            .unwrap();
    }

    #[test]
    fn get_data2() {
        let connection = sqlite::open("data/sqlite3/test.sql").unwrap();
        let query = "SELECT * FROM data WHERE age > ?";
        let mut statement = connection.prepare(query).unwrap();
        statement.bind((1, 15)).unwrap();

        while let Ok(State::Row) = statement.next() {
            println!("name = {}", statement.read::<String, _>("name").unwrap());
            println!("age = {}", statement.read::<i64, _>("age").unwrap());
        }
    }

    #[test]
    fn get_table_info() {
        let connection = sqlite::open("data/sqlite3/CWB_20250109.sql").unwrap();
        let query = "PRAGMA table_info(cwb_meteo_dy);";
        let mut statement = connection.prepare(query).unwrap();

        while let Ok(State::Row) = statement.next() {
            println!("name = {}", statement.read::<String, _>("name").unwrap());
            println!("type = {}", statement.read::<String, _>("type").unwrap());
        }

        // connection
        //     .iterate(query, |datas| {
        //         for &(name, value) in datas.iter() {
        //             println!("{name}: {value:?}");
        //         }
        //         true
        //     })
        //     .unwrap();
    }

    #[test]
    fn do_test() {
        let re = Regex::new(r"CWB_[0-9]{8}.sql").unwrap();
        let root= "./data/sqlite3";
        let filename = get_last_modified_file(root, re).unwrap();

        let path = Path::new(&root).join(filename);
        println!("path: {:?}", path);

        println!("filename: {:?}", path.file_name());
        println!("file_stem: {:?}", path.file_stem());
        println!("extension: {:?}", path.extension());

    }
}
