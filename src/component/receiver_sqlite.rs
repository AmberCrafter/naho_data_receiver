use std::{
    error::Error,
    fs::create_dir_all,
    path::Path,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use chrono::NaiveDateTime;

const TABLE_NAME: &str = "data";
const TABLE_INFO: &str = "
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    datetime TEXT,
    dkind TEXT,
    value TEXT
";

fn create_db<P>(path: P) -> Result<(), Box<dyn Error + 'static>>
where
    P: AsRef<Path>,
{
    let statement = format!("create table if not exists {TABLE_NAME} ({TABLE_INFO});");
    let connection = sqlite::open(path)?;
    connection.execute(statement)?;

    Ok(())
}

pub fn setup_sqlite3_recorder(
    receiver: Receiver<String>,
    root: &str,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    create_dir_all(root)?;
    let root = root.to_string();

    let handler = thread::spawn(move || {
        loop {
            while let Ok(msg) = receiver.recv() {
                let mut words = msg.split(',');

                let Some(dkind) = words.nth(2) else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let Some(timestr) = words.next() else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let Ok(time) = NaiveDateTime::parse_from_str(timestr, "%Y%m%d%H%M") else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let filename = format!("CWB_{}.sql", time.format("%Y%m%d"));
                let filepath = Path::new(&root).join(filename);

                if !filepath.exists() {
                    match create_db(&filepath) {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("Create database failed: {e} - {filepath:?} - {msg}");
                            continue;
                        }
                    }
                }

                if let Ok(connection) = sqlite::open(filepath) {
                    let statement = format!(
                        "INSERT into {TABLE_NAME} (datetime, dkind, value) values ('{}', '{}', '{}');",
                        time.format("%Y-%m-%d %H:%M:%S"),
                        dkind,
                        msg
                    );
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
