use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::Write,
    path::Path,
    sync::mpsc::Receiver,
    thread::{self, JoinHandle},
};

pub fn setup_rawdata_recorder(
    receiver: Receiver<String>,
    root: &str,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    create_dir_all(root)?;
    let root = root.to_string();

    let handler = thread::spawn(move || {
        loop {
            while let Ok(msg) = receiver.recv() {
                let filepath = Path::new(&root).join("data.txt");
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
