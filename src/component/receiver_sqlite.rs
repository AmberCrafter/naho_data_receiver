use std::{
    collections::{HashMap, HashSet},
    error::Error,
    fmt::Display,
    fs::create_dir_all,
    path::Path,
    sync::{mpsc::Receiver, Arc},
    thread::{self, JoinHandle},
};

use crate::{component::{backup_file, is_update_header, receiver_raw::gen_headertable_key}, config::SystemConfig};
use chrono::{NaiveDateTime, NaiveTime};
use regex::Regex;

use super::{
    codec::{CodecConfigBase, CodecConfigMetadata}, generate_db_filepath, HeaderTable, MsgPayload, DTAETIME_FMT
};

#[allow(unused)]
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
    let Ok(dir) = std::fs::read_dir(&root) else {
        log::error!("Couldn't access local directory");
        return None;
    };
    let Some(last_modified_file) = dir
        .flatten() // Remove failed
        .filter(|f| f.metadata().unwrap().is_file()) // Filter out directories (only consider files)
        .filter(|f| re.is_match(f.file_name().to_str().unwrap())) // Filter out directories (only consider files)
        .max_by_key(|x| x.metadata().unwrap().modified().unwrap())
    // Get the most recently modified file
    else {
        return None;
    };

    if let Some(filename) = last_modified_file.file_name().to_str() {
        Some(filename.to_string())
    } else {
        None
    }
}

fn create_db<P>(path: P, config: &CodecConfigBase) -> Result<(), Box<dyn Error + 'static>>
where
    P: AsRef<Path>,
{
    let connection = sqlite::open(path)?;
    for mem in config.metadatas.iter() {
        for dkind in mem.dkind.iter() {
            if let Some(statement) = config.gen_sqlite3_create_table_cmd(dkind, &mem.name) {
                connection.execute(statement)?;
            }
        }
    }

    let mut tableinfo = String::new();
    tableinfo.push_str("id INTEGER PRIMARY KEY AUTOINCREMENT");
    tableinfo.push_str(", tablename TEXT");
    tableinfo.push_str(", header TEXT");

    let statement = format!("CREATE TABLE IF NOT EXISTS headers ({tableinfo});");
    connection.execute(statement)?;
    Ok(())
}

fn check_column<P>(path: P, config: &CodecConfigBase) -> Result<(), SQLiteErrorType>
where
    P: AsRef<Path>,
{
    let Ok(connection) = sqlite::open(path) else {
        return Err(SQLiteErrorType::Invalid);
    };
    for dconfig in config.metadatas.iter() {
        let mut set = HashSet::new();
        let query = format!("PRAGMA table_info({});", dconfig.name);
        let Ok(mut statement) = connection.prepare(query) else {
            return Err(SQLiteErrorType::Invalid);
        };
        while let Ok(sqlite::State::Row) = statement.next() {
            if let Ok(name) = statement.read::<String, _>("name") {
                set.insert(name);
            }
        }

        for cfg_name in &dconfig.formation {
            if !set.contains(&cfg_name.sqlite3.name) {
                return Err(SQLiteErrorType::NotMatch);
            }
        }
    }
    Ok(())
}

fn parse_rawdata(
    rawdata: &str,
    config: &CodecConfigMetadata,
) -> (Option<String>, Option<NaiveDateTime>) {
    let mut result = (None, None);
    let mut buf = Vec::new();
    let mut words = rawdata.split(',');

    // stx
    if config.stx_etx == Some(true) && words.next() != Some("\u{2}") {
        return result;
    }

    for dtype in config.formation.iter() {
        let Some(subdata) = words.next() else {
            log::error!("System error!");
            return result;
        };

        match (dtype.sqlite3.dtype.as_str(), dtype.sqlite3.unit.as_deref()) {
            ("TEXT", Some("YYYY-mm-dd HH:MM:SS")) => {
                let Some(formation) = &dtype.rust.unit else {
                    log::error!("Invalid data!");
                    return result;
                };

                let Ok(temp) = NaiveDateTime::parse_from_str(subdata, formation) else {
                    log::error!("Invalid data!");
                    return result;
                };
                buf.push(temp.format("'%Y-%m-%d %H:%M:%S'").to_string());
                if dtype.rust.major_datetime == Some(true) {
                    result.1.replace(temp.clone());
                }
            }
            ("TEXT", Some("HH:MM:SS")) => {
                let Some(formation) = &dtype.rust.unit else {
                    log::error!("Invalid data!");
                    return result;
                };

                let Ok(temp) = NaiveTime::parse_from_str(subdata, formation) else {
                    log::error!("Invalid data! {subdata}");
                    return result;
                };
                buf.push(temp.format("'%H:%M:%S'").to_string());
            }
            ("TEXT", _) => buf.push(format!("'{}'", subdata)),
            _ => buf.push(subdata.to_string()),
        }
    }

    // etx
    if config.stx_etx == Some(true) && words.next() != Some("\u{3}") {
        return result;
    }
    result.0.replace(buf.join(","));
    result
}

fn check_sqlfile(config: &SystemConfig) {
    for (key, val) in config.codec.iter() {
        let Some(cfg_sqlite3) = val.sqlite3.as_ref() else {
            log::info!("Unsupport sqlite3 recorder: {:?}", key);
            continue;
        };

        let Some(regexp) = cfg_sqlite3.regex.as_ref() else {
            log::info!("Unsupport sqlite3 precheck: {:?}", key);
            continue;
        };

        let Ok(re) = Regex::new(&regexp) else {
            log::error!("System error: {:?}", key);
            continue;
        };

        let Some(lastfile) = get_last_modified_file(&cfg_sqlite3.directory, re) else {
            log::info!("Last modify file not found: {:?}", key);
            continue;
        };

        let filepath = Path::new(&cfg_sqlite3.directory).join(lastfile);
        match check_column(&filepath, &val) {
            Err(SQLiteErrorType::NotMatch) => {
                log::info!("check_column: {:?}", SQLiteErrorType::NotMatch);
                if let Err(e) = backup_file(&filepath) {
                    log::error!("System Error. {e}");
                }
            }
            Ok(()) => {}
            ret => {
                log::error!("{:?}", ret);
            }
        }
    }
}

pub fn setup_sqlite3_recorder(
    receiver: Receiver<Arc<MsgPayload>>,
    config: Arc<SystemConfig>,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    for (_key, val) in config.codec.iter() {
        if let Some(cfg) = val.sqlite3.as_ref() {
            create_dir_all(&cfg.directory)?;
        }
    }

    let handler = thread::spawn(move || {
        let mut header_table = HeaderTable::new();
        check_sqlfile(&config);

        loop {
            while let Ok(msg) = receiver.recv() {
                // for header msg
                if msg.update_header {
                    let key = gen_headertable_key(&msg);
                    if is_update_header(&mut header_table, &key, &msg.value) {
                        check_sqlfile(&config);
                    }
                    continue;
                }

                // for data msg
                let Some(cfg) = config.codec.get(&msg.tag) else {
                    log::error!("Unsupport tag: {:?}", msg.tag);
                    continue;
                };

                let Some(cfg_sqlite3) = cfg.sqlite3.as_ref() else {
                    log::info!("Unsupport record rawdata: {:?}", msg.tag);
                    continue;
                };

                let Some(dconfig) = cfg.get_data_config(&msg.dkind) else {
                    log::error!("Invalid: {msg:?}");
                    continue;
                };

                let columnname = dconfig
                    .formation
                    .iter()
                    .map(|mem| mem.sqlite3.name.to_string())
                    .collect::<Vec<_>>()
                    .join(",");

                for value in msg.value.iter() {
                    let (Some(data_str), Some(time)) = parse_rawdata(value, dconfig) else {
                        log::error!("Invalid: {msg:?}");
                        continue;
                    };

                    let mut opts = HashMap::new();
                    opts.insert(
                        "datetime".to_string(),
                        time.format(DTAETIME_FMT).to_string(),
                    );
                    let filepath =
                        generate_db_filepath(&cfg.tag, cfg_sqlite3, dconfig, &opts).unwrap();

                    if let Some(root) = filepath.parent() {
                        if let Err(e) = create_dir_all(&root) {
                            log::error!("System Error. {e}");
                        }
                    }

                    if !filepath.exists() {
                        match create_db(&filepath, &cfg) {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("Create database failed: {e} - {filepath:?} - {msg:?}");
                                continue;
                            }
                        }
                    }

                    if let Ok(connection) = sqlite::open(filepath) {
                        let statement = if dconfig.raw_save == Some(true) {
                            format!(
                                "INSERT into {} ({},rawdata) values ({},'{}');",
                                &dconfig.name, columnname, data_str, value
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
                        log::error!("Open database failed: {msg:?}");
                        continue;
                    }
                }
            }
        }
    });
    Ok(handler)
}

#[cfg(test)]
mod test {
    use sqlite::State;

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
}
