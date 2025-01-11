use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

use chrono::NaiveDateTime;

pub fn setup_rawdata_recorder(
    receiver: Receiver<String>,
    root: &str,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    create_dir_all(root)?;
    let root = root.to_string();

    let handler = thread::spawn(move || {
        loop {
            while let Ok(msg) = receiver.recv() {
                let mut words = msg.split(',');

                let Some(timestr) = words.nth(3) else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let Ok(time) = NaiveDateTime::parse_from_str(timestr, "%Y%m%d%H%M") else {
                    log::error!("Invalid: {msg}");
                    continue;
                };

                let filename = format!("CWB_{}.txt", time.format("%Y%m%d"));
                let filepath = Path::new(&root).join(filename);
                let mut file = File::options()
                    .create(true)
                    .append(true)
                    .open(filepath)
                    .expect("Create raw data file failed");
                file.write(format!("{}\n", msg).as_bytes())
                    .expect("Write failed");
            }
        }
        0
    });
    Ok(handler)
}
