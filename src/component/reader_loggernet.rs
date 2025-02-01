use std::{
    error::Error,
    fs::{create_dir_all, rename, File},
    io::{BufRead, BufReader},
    path::Path,
    sync::{mpsc::Sender, Arc},
    thread::{self, sleep, JoinHandle},
    time::Duration,
};

use crate::config::SystemConfig;

use super::{is_update_header, HeaderTable, MsgPayload};

const TEMP_FOLDER: &str = "./data/temp";

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

    if create_dir_all(TEMP_FOLDER).is_err() {
        log::error!("System Error. Unable to ceate temperal folder.Permission deny.");
        return Err(
            String::from("System Error. Unable to ceate temperal folder.Permission deny.").into(),
        );
    }

    let handle = thread::spawn(move || {
        loop {
            // 1. move original file
            // 2. send data
            for listen_target in listen_list.iter() {
                let listen_path = Path::new(&listen_target.path);
                let Some(filename) = listen_path.file_name() else {
                    log::error!("System Error. listen_path: {listen_path:?}");
                    continue;
                };
                let Some(filename) = filename.to_str() else {
                    log::error!("System Error. listen_path: {listen_path:?}");
                    continue;
                };
                let tmp_path = Path::new(TEMP_FOLDER).join(filename);

                if !listen_path.exists() {
                    continue;
                }
                if let Err(e) = rename(listen_path, &tmp_path) {
                    log::error!("System Error: {e}");
                    continue;
                }

                let mut buffer = String::new();
                let file = match File::open(&tmp_path) {
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
            }
            sleep(Duration::from_secs(5));
        }
    });
    Ok(handle)
}
