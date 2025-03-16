use std::{
    error::Error,
    fs::{remove_file, rename, File},
    io::{BufRead, BufReader},
    path::Path,
    sync::{mpsc::Sender, Arc},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use crate::{component::utils::files::is_update_header, config::SystemConfig};

use super::{HeaderTable, MsgPayload};

pub fn setup_file_listen_naho(
    config: Arc<SystemConfig>,
    sender: Sender<Arc<MsgPayload>>,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    let mut listen_list = Vec::new();
    let mut header_table = HeaderTable::new();

    let Some(cfg_listen_list) = config.global.listen_list.as_ref() else {
        log::info!("global.listen_list is empty.");
        return Err(String::from("global.listen_list is empty.").into());
    };

    for val in cfg_listen_list {
        if val.tag == "NAHO" {
            listen_list.push(val.clone());
        }
    }

    if listen_list.is_empty() {
        log::info!("No listen target with tag: NAHO.");
        return Err(String::from("No listen target with tag: NAHO.").into());
    }

    let handle = thread::spawn(move || {
        loop {
            // 1. move original file
            // 2. send data
            for listen_target in listen_list.iter() {
                let cfg_listen_path = Path::new(&listen_target.path);
                let Some(filename) = cfg_listen_path.file_name() else {
                    log::error!("System Error. cfg_listen_path: {cfg_listen_path:?}");
                    continue;
                };
                let Some(filename) = filename.to_str() else {
                    log::error!("System Error. cfg_listen_path: {cfg_listen_path:?}");
                    continue;
                };

                if !cfg_listen_path.exists() {
                    continue;
                }

                let mut listen_file = cfg_listen_path.to_string_lossy().to_string();
                let mut do_remove_file = false;
                if let Some(flags) = &listen_target.flags {
                    if flags.f_move == Some(true) {
                        let suffix =
                            if let Some(suffix) = config.global.listen_move_suffix.as_deref() {
                                suffix
                            } else {
                                "lock"
                            };

                        listen_file = format!("{listen_file}.{suffix}");
                        if let Err(e) = rename(cfg_listen_path, &listen_file) {
                            log::error!("System Error: {e}");
                            continue;
                        }
                    }

                    if flags.f_remove_after_used == Some(true) {
                        do_remove_file = true;
                    }
                }

                let mut buffer = String::new();
                let file = match File::open(&listen_file) {
                    Ok(file) => file,
                    Err(e) => {
                        log::error!("System Error. {e}");
                        continue;
                    }
                };
                let mut reader = BufReader::new(file);

                if let Some(cfg_header) = listen_target.header.as_ref() {
                    let number = cfg_header.number;
                    let mut header = Vec::new();
                    // header
                    for _ in 0..number {
                        match reader.read_line(&mut buffer) {
                            Ok(num) => {
                                if num == 0 {
                                    break;
                                }
                            }
                            Err(e) => {
                                log::error!("System Error. {e}");
                                break;
                            }
                        }
                        header.push(buffer.to_string());
                        buffer.clear();
                    }

                    // error is recorded in previous for loop
                    if header.len() < number {
                        continue;
                    }

                    if is_update_header(&mut header_table, filename, &header) {
                        // consume header
                        let mut msg =
                            MsgPayload::new(&listen_target.tag, &listen_target.dkind, header);
                        if let Err(e) = msg.set_update_header() {
                            log::error!("Setup update header flag failed: {e}");
                        }

                        if let Err(e) = sender.send(Arc::new(msg)) {
                            log::error!("Send header failed: {e}");
                        }
                    }
                }

                buffer.clear();
                let mut counter = 0;
                let mut values = Vec::new();
                while let Ok(num) = reader.read_line(&mut buffer) {
                    if num == 0 {
                        break;
                    }
                    if counter >= 100 {
                        let msg = MsgPayload::new(&listen_target.tag, &listen_target.dkind, values);
                        if let Err(e) = sender.send(Arc::new(msg)) {
                            log::error!("Send data failed: {e}");
                        }
                        values = Vec::new();
                        counter = 0;
                    }

                    values.push(buffer.clone());
                    buffer.clear();
                    counter += 1;
                }

                let msg = MsgPayload::new(&listen_target.tag, &listen_target.dkind, values);
                if let Err(e) = sender.send(Arc::new(msg)) {
                    log::error!("Send data failed: {e}");
                }

                if do_remove_file {
                    if let Err(e) = remove_file(&listen_file) {
                        log::error!("Remove file failed: {e}");
                    }
                }
                log::info!(target: "info", "Listened {}", &listen_file);
            }
            sleep(Duration::from_secs(5));
        }
    });
    Ok(handle)
}
