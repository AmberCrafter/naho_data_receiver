use std::{
    collections::HashMap, error::Error, fs::{create_dir_all, File}, io::Write, path::Path, sync::{mpsc::Receiver, Arc}, thread::{self, JoinHandle}
};

use chrono::NaiveDateTime;

use crate::{component::{backup_file, HeaderTableValue}, config::SystemConfig};

use super::{cal_hash, generate_db_filepath, HeaderTable, MsgPayload, DTAETIME_FMT};

fn gen_headertable_key(msg: &MsgPayload) -> String
{
    format!("{}_{}", msg.tag, msg.dkind)
}

fn create_file_with_header<P>(path: P, msg: &MsgPayload, table: &mut HeaderTable) -> Result<(), Box<dyn Error + 'static>>
where
    P: AsRef<Path>
{
    fn _create_file_with_header<P>(path: P, header: &Vec<String>) -> Result<(), Box<dyn Error + 'static>>
    where
        P: AsRef<Path>
    {
        if let Some(root) = path.as_ref().parent() {
            create_dir_all(&root)?;
        }

        let mut file = File::create(path)?;
        for val in header {
            file.write(val.as_bytes())?;
        }
        Ok(())
    }
    
    match (msg.tag.as_str(), msg.dkind.as_str()) {
        ("NAHO", _dkind) => {
            let key = gen_headertable_key(msg);
            let Some(tval) = table.get_mut(&key) else {
                return Err(String::from("Can't find header.").into());
            };

            if !path.as_ref().exists() {
                _create_file_with_header(&path, &tval.header)?;
            } else if tval.is_update {
                backup_file(&path)?;
                _create_file_with_header(&path, &tval.header)?;
            }
            tval.is_update = false;
        }
        _ => {
            if !path.as_ref().exists() {
                _create_file_with_header(path,&Vec::new())?;
            }
        }
    }
    Ok(())
}


pub fn setup_rawdata_recorder(
    receiver: Receiver<Arc<MsgPayload>>,
    config: Arc<SystemConfig>,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    for (_key, val) in config.codec.iter() {
        if let Some(cfg) = val.rawdata.as_ref() {
            create_dir_all(&cfg.directory)?;
        }
    }

    let handler = thread::spawn(move || {
        let mut header_table = HeaderTable::new();

        loop {
            while let Ok(msg) = receiver.recv() {
                // for header msg
                if msg.update_header {
                    let new_hash = cal_hash(&msg.value);

                    let key = gen_headertable_key(&msg);
                    header_table.entry(key)
                        .and_modify(|tval| {
                            if tval.hash != new_hash {
                                tval.header = msg.value.clone();
                                tval.hash = new_hash;
                                tval.is_update = true;
                            }
                        })
                        .or_insert(HeaderTableValue {
                            hash: new_hash,
                            header: msg.value.clone(),
                            is_update: true
                        });

                    continue;
                }

                // for data msg
                let Some(cfg) = config.codec.get(&msg.tag) else {
                    log::error!("Unsupport tag: {:?}", msg.tag);
                    continue;
                };

                let Some(cfg_rawdata) = cfg.rawdata.as_ref() else {
                    log::info!("Unsupport record rawdata: {:?}", msg.tag);
                    continue;
                };

                let Some(dconfig) = cfg.get_data_config(&msg.dkind) else {
                    log::error!("Invalid: {msg:?}");
                    continue;
                };

                let Some(datetime_info) = dconfig.get_datetime_info() else {
                    log::error!(
                        "Unsupport data format. tag:{:?}; dkind:{:?}",
                        &msg.tag,
                        &msg.dkind
                    );
                    continue;
                };


                for value in msg.value.iter() {
                    let mut words = value.split(',');
    
                    let offset = if dconfig.stx_etx==Some(true) {
                        datetime_info.0 + 1
                    } else {
                        datetime_info.0
                    };
                    let Some(timestr) = words.nth(offset) else {
                        log::error!("Invalid: {value:?}");
                        continue;
                    };
    
                    let Some(timefmt) = &datetime_info.1.rust.unit else {
                        log::error!(
                            "Unsupport data format. tag:{:?}; dkind:{:?}",
                            &msg.tag,
                            &msg.dkind
                        );
                        continue;
                    };
    
                    let Ok(time) = NaiveDateTime::parse_from_str(timestr, &timefmt) else {
                        log::error!("Invalid: {value:?}");
                        continue;
                    };
    
                    let mut opts = HashMap::new();
                    opts.insert("datetime".to_string(), time.format(DTAETIME_FMT).to_string());
                    let filepath = generate_db_filepath(&cfg.tag, cfg_rawdata, dconfig, &opts).unwrap();
                    
                    if let Err(e) = create_file_with_header(&filepath, &msg, &mut header_table) {
                        log::error!("System Error. {e}");
                    }
                    
                    let mut file = match File::options()
                        .append(true)
                        .open(&filepath) 
                    {
                        Ok(file) => file,
                        Err(e) => {
                            log::error!("System Error. {e}");
                            continue;
                        }
                    };

                    if let Err(e) = file.write(format!("{}\n", value.trim()).as_bytes()){
                        log::error!("System Error. {e}");
                    }
                }
            }
        }
    });
    Ok(handler)
}
