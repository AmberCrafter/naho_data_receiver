use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use chrono::NaiveDateTime;
use serde::Deserialize;
use serde_json::{json, Map, Value};

const FILTER_HEADER_ST: &str = "Start time(LT)";
const FILTER_HEADER_ET: &str = "End time(LT)";
const FILTER_HEADER_FLAG: &str = "Flag";
const FILTER_DATETIME_FMT: &str = "\"%Y-%m-%d %H:%M:%S\"";

const CFG_LEVEL: &str = "Level";
const CFG_SOURCE: &str = "source";

type FilterFlagTable = HashMap<String, Value>;
type FilterRuleTable = HashMap<String, Vec<FilterRule>>;

#[derive(Debug)]
struct FilterRule {
    starttime: NaiveDateTime,
    endtime: NaiveDateTime,
    target: String,
}

#[derive(Debug, Default)]
struct Filter {
    flags: FilterFlagTable,
    rules: FilterRuleTable,
}

impl Filter {
    pub fn new() -> Self {
        Filter::default()
    }

    fn load_flag_table<P>(&mut self, path: P) -> Result<(), Box<dyn Error + 'static>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        self.flags = serde_json::from_reader(reader)?;
        Ok(())
    }

    fn get_level(&self, level: usize) -> Option<&Map<String, Value>> {
        let tag = format!("{CFG_LEVEL}{level}");
        self.flags.get(&tag)?.as_object()
    }

    fn get_source(&self, level: usize) -> Option<&str> {
        self.get_level(level)?.get(CFG_SOURCE)?.as_str()
    }

    fn get_flags(&self, level: usize, flag: usize) -> Option<&Vec<Value>> {
        let tag = format!("{flag}");
        self.get_level(level)?.get(&tag)?.as_array()
    }

    pub fn load<P>(&mut self, path: P) -> Result<(), Box<dyn Error + 'static>>
    where
        P: AsRef<Path>,
    {
        self.load_flag_table(path)?;
        
        for cfg in self.flags.iter() {
            let Some(level_cfg) = cfg.1.as_object() else {
                continue;
            };
            let Some(source) = level_cfg.get(CFG_SOURCE) else {
                continue;
            };
            let Some(src) = source.as_str() else {
                continue;
            };

            let file = File::open(src)?;
            let mut reader = BufReader::new(file);
            let mut buffer = String::new();
    
            // find header
            while let Ok(_nums) = reader.read_line(&mut buffer) {
                if buffer.find(FILTER_HEADER_ST).is_some()
                    && buffer.find(FILTER_HEADER_ET).is_some()
                    && buffer.find(FILTER_HEADER_FLAG).is_some()
                {
                    // println!("{buffer}");
                    break;
                }
                buffer.clear();
            }
            buffer.clear();
    
            while let Ok(num) = reader.read_line(&mut buffer) {
                if num==0 {break;}
                // println!("{buffer}");

                let mut words = buffer.split(',');
                let st = if let Some(buf) = words.next() {
                    let Ok(st) = NaiveDateTime::parse_from_str(buf, FILTER_DATETIME_FMT) else {
                        log::error!("System Error: {}", buffer);
                        buffer.clear();
                        continue;
                    };
                    st
                } else {
                    log::error!("System Error: {}", buffer);
                    buffer.clear();
                    continue;
                };
    
                let et = if let Some(buf) = words.next() {
                    let Ok(et) = NaiveDateTime::parse_from_str(buf, FILTER_DATETIME_FMT) else {
                        log::error!("System Error: {}", buffer);
                        buffer.clear();
                        continue;
                    };
                    et
                } else {
                    log::error!("System Error: {}", buffer);
                    buffer.clear();
                    continue;
                };
    
                let Some(buf) = words.next() else {
                    log::error!("System Error: {}", buffer);
                    buffer.clear();
                    continue;
                };

                let rule = FilterRule { starttime: st, endtime: et, target: buf.to_string() };
                let entry = self.rules.entry(cfg.0.to_string())
                    .or_insert(Vec::new());
                entry.push(rule);
                buffer.clear();
            }
        }
        Ok(())
    }
}


#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn case1() {
        let path = "./config/filter.json";
        let mut cfg = Filter::new();
        cfg.load(path);
        println!("{:?}", cfg);
        println!("{:?}", cfg.get_source(1));
        println!("{:?}", cfg.get_flags(1, 11));
        println!("{:?}", cfg.get_flags(1, 41));
        println!("{:?}", cfg.get_flags(1, 43));
    }
}