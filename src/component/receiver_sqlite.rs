use std::{
    error::Error,
    fs::create_dir_all,
    path::Path,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use chrono::{NaiveDateTime, NaiveTime};

use crate::component::parser_cwb::CWBCodecConfig;

use super::parser_cwb::CWBDataConfig;

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

        match dtype.rust.dtype.as_str() {
            "NaiveDateTime" => {
                let Ok(temp) = NaiveDateTime::parse_from_str(subdata, "%Y%m%d%H%M") else {
                    log::error!("Invalid data!");
                    return None;
                };
                buf.push(temp.format("'%Y-%m-%d %H:%M:%S'").to_string());
            }
            "NaiveTime" => {
                let Ok(temp) = NaiveTime::parse_from_str(subdata, "%H%M") else {
                    log::error!("Invalid data!");
                    return None;
                };
                buf.push(temp.format("'%H:%M:%S'").to_string());
            }
            "String" => buf.push(format!("'{}'", subdata)),
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
}
