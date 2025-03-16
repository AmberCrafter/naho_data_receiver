#![allow(unused)]
mod filter;

use std::{error::Error, path::Path};

pub fn sqlite_get_columns<P>(path: P, table: &str) -> Result<Vec<String>, Box<dyn Error + 'static>>
where
    P: AsRef<Path>,
{
    let conn = sqlite::open(path)?;
    let query = format!("select * from {table} limit 0;");
    let stat = conn.prepare(query)?;
    Ok(stat.column_names().to_vec())
}

pub fn sqlite_dedup_and_sort_by<P, F>(
    path: P,
    table: &str,
    dedupe_col: &str,
    sort_col: &str,
    callback: F,
) -> Result<(), Box<dyn Error + 'static>>
where
    P: AsRef<Path>,
    F: FnMut(&[(&str, Option<&str>)]) -> bool,
{
    let conn = sqlite::open(path)?;

    let query = format!(
        "select {table}.* from {table}
        inner join (
            select max(id) as max_id from {table} group by {dedupe_col}
        ) group_table
        ON group_table.max_id = {table}.id
        order by {table}.{sort_col}"
    );
    conn.iterate(query, callback)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::component::qc_level1::sqlite_get_columns;

    use super::sqlite_dedup_and_sort_by;

    #[test]
    fn case1() {
        let conn = sqlite::open("data/sqlite3/cwb/CWB_20250109.sql").unwrap();

        // // Show tables
        // let query = "select * from sqlite_master where type='table';";
        // conn.iterate(query, |datas| {
        //     for (key, value) in datas.iter() {
        //         println!("{:?}: {:?}", key, value);
        //     }
        //     true
        // }).unwrap();

        //
        // let query = "select * from cwb_meteo_dy where dtime='2025-01-09 15:55:00';";
        // let query = "select cwb_meteo_dy.* from cwb_meteo_dy
        //     inner join (
        //         select max(id) as maxid, dtime from cwb_meteo_dy group by dtime
        //     ) group_table
        //     ON group_table.maxid = cwb_meteo_dy.id
        //     where cwb_meteo_dy.dtime='2025-01-09 15:55:00';";

        let query = "select cwb_meteo_dy.* from cwb_meteo_dy 
            inner join (
                select max(id) as maxid, dtime from cwb_meteo_dy group by dtime
            ) group_table 
            ON group_table.maxid = cwb_meteo_dy.id
            order by cwb_meteo_dy.dtime;";

        // let mut statement = conn.prepare(query).unwrap();
        // while let Ok(State::Row) = statement.next() {
        //     println!("{:?}", statement.read::<String, _>("id").unwrap());
        // }
        conn.iterate(query, |datas| {
            println!("{:?}", datas);
            // for row in datas.iter() {
            //     println!("{:?}", row);
            // }
            true
        })
        .unwrap();
    }

    #[test]
    fn case2() {
        let conn = sqlite::open("data/sqlite3/naho/NAHO_20250119.sql").unwrap();
        let query = "select ori.* from CR1000XSeries_Datatable_Sec ori 
            inner join (
                select max(id) as maxid, timestamp from CR1000XSeries_Datatable_Sec group by timestamp
            ) group_table 
            ON group_table.maxid = ori.id
            order by ori.timestamp;";

        // let mut statement = conn.prepare(query).unwrap();
        // while let Ok(State::Row) = statement.next() {
        //     println!("{:?}", statement.read::<String, _>("id").unwrap());
        // }
        conn.iterate(query, |datas| {
            println!("{:?}", datas.iter().nth(0));
            // for row in datas.iter() {
            //     println!("{:?}", row);
            // }
            true
        })
        .unwrap();
    }

    #[test]
    fn case3() {
        let mut buffer = Vec::new();
        // let cb = |row: &[(&str, Option<&str>)]| {
        //     let tmp = row.iter().map(|(_key, val)| if let Some(v) = val {v.to_string()} else {"null".to_string()}).collect::<Vec<_>>();
        //     buffer.push(tmp);
        //     // buffer.push(row);
        //     true
        // };

        let path = "data/sqlite3/naho/NAHO_20250119.sql";
        let res = sqlite_dedup_and_sort_by(
            path,
            "CR1000XSeries_Datatable_Sec",
            "timestamp",
            "timestamp",
            |row| {
                let tmp = row
                    .iter()
                    .map(|(_key, val)| {
                        if let Some(v) = val {
                            v.to_string()
                        } else {
                            "null".to_string()
                        }
                    })
                    .collect::<Vec<_>>();
                buffer.push(tmp);
                true
            },
        );

        println!("{:#?}", buffer.get(0..100));
    }

    #[test]
    fn case4() {
        let path = "data/sqlite3/naho/NAHO_20250119.sql";
        let res = sqlite_get_columns(path, "CR1000XSeries_Datatable_Sec");

        println!("{:#?}", res);
    }
}
