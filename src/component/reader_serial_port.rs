use std::{
    error::Error, io::{self, BufRead, BufReader, Read}, sync::mpsc::Sender, thread::{self, sleep, JoinHandle}, time::Duration
};

pub fn setup_serial_port(
    path: &str,
    baudrate: u32,
    sender: Sender<String>
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    let uart = serialport::new(path, baudrate)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Open serial port failed");

    let handle = thread::spawn(move || {
        let mut buf = String::new();
        let mut reader = BufReader::new(uart);

        loop {
            buf.clear();
            match reader.read_line(&mut buf) {
                Ok(num) => {
                    let msg = buf.trim();
                    log::info!("[{}] {}", num, msg);
                    if let Err(e) = sender.send(msg.to_string()) {
                        log::error!("{e}");
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => log::error!("{e}"),
            };
            // println!("sleep...");
            // sleep(Duration::from_millis(50));
        }
        
        0
    });

    Ok(handle)
}

pub fn setup_serial_port_cwb(
    path: &str,
    baudrate: u32,
    sender: Sender<String>
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    let uart = serialport::new(path, baudrate)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Open serial port failed");

    let handle = thread::spawn(move || {
        let mut buffer = Vec::new();
        let mut reader = BufReader::new(uart);

        loop {
            buffer.clear();
            match reader.read_until(0x3, &mut buffer) {
                Ok(num) => {
                    log::info!(target: "serialport", "[{}] {:?}", num, buffer);
                    let Some(stx_idx) = buffer.iter().position(|&ele| ele==0x2)
                        else { 
                            log::warn!("Invalid data: {buffer:?}");
                            continue;
                        };
                    let (_, rawmsg) = buffer.split_at_mut(stx_idx);

                    let msg = String::from_utf8_lossy(rawmsg).to_string();
                    log::info!(target: "console", "[{}] {:?}", num, msg);
                    if let Err(e) = sender.send(msg) {
                        log::error!("{e}");
                    }
                },
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => log::error!("{e}"),
            };
            // println!("sleep...");
            // sleep(Duration::from_millis(50));
        }
        
        0
    });

    Ok(handle)
}
