use std::{
    error::Error,
    io::{self, BufRead, BufReader},
    sync::{mpsc::Sender, Arc},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::component::{parser_cwb::get_dkind, MsgPayload};

// pub fn setup_serial_port(
//     path: &str,
//     baudrate: u32,
//     sender: Sender<String>
// ) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
//     let uart = serialport::new(path, baudrate)
//         .timeout(Duration::from_millis(100))
//         .open()
//         .expect("Open serial port failed");

//     let handle = thread::spawn(move || {
//         let mut buf = String::new();
//         let mut reader = BufReader::new(uart);

//         loop {
//             buf.clear();
//             match reader.read_line(&mut buf) {
//                 Ok(num) => {
//                     let msg = buf.trim();
//                     log::info!("[{}] {}", num, msg);
//                     if let Err(e) = sender.send(msg.to_string()) {
//                         log::error!("{e}");
//                     }
//                 },
//                 Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
//                 Err(e) => log::error!("{e}"),
//             };
//             // println!("sleep...");
//             // sleep(Duration::from_millis(50));
//         }
//     });

//     Ok(handle)
// }


// This function is better than `setup_serial_port_cwb()` which can check crc value
#[allow(dead_code)]
pub fn setup_serial_port_cwb_by_line(
    path: &str,
    baudrate: u32,
    sender: Sender<MsgPayload>,
) -> Result<JoinHandle<usize>, Box<dyn Error + 'static>> {
    let uart = serialport::new(path, baudrate)
        .timeout(Duration::from_millis(100))
        .open()
        .expect("Open serial port failed");

    let handle = thread::spawn(move || {
        let mut buffer = String::new();
        let mut reader = BufReader::new(uart);

        loop {
            buffer.clear();
            match reader.read_line(&mut buffer) {
                Ok(num) => {
                    log::info!(target: "serialport", "[{}] {:?}", num, buffer);
                    log::info!(target: "console", "[{}] {:?}", num, buffer);
                    let Some(etx_idx) = buffer.find('\u{3}') else {
                        log::warn!("Invalid data: {buffer:?}");
                        continue;
                    };
                    let (msg, checksum) = buffer.split_at_mut(etx_idx + 1);

                    if let Some(_stx_idx) = checksum.find('\u{2}') {
                        log::error!("Invalid data: {buffer:?}");
                        continue;
                    }

                    let Some(dkind) = get_dkind(msg) else {
                        log::error!("Invalid data: {msg:?}");
                        continue;
                    };

                    let msg = vec![msg.to_string()];
                    let payload = MsgPayload::new("CWB", &dkind, msg);
                    if let Err(e) = sender.send(payload) {
                        log::error!("{e}");
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => log::error!("{e}"),
            };
            // println!("sleep...");
            // sleep(Duration::from_millis(50));
        }
    });

    Ok(handle)
}

pub fn setup_serial_port_cwb(
    path: &str,
    baudrate: u32,
    sender: Sender<Arc<MsgPayload>>,
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
                // match reader.read_until(0x10, &mut buffer) {
                Ok(num) => {
                    log::info!(target: "serialport", "[{}] {:?}", num, buffer);
                    let Some(stx_idx) = buffer.iter().position(|&ele| ele == 0x2) else {
                        log::warn!("Invalid data: {buffer:?}");
                        continue;
                    };
                    let (_, rawmsg) = buffer.split_at_mut(stx_idx);

                    let msg = String::from_utf8_lossy(rawmsg).to_string();
                    log::info!(target: "console", "[{}] {:?}", num, msg);
                    log::info!(target: "serialport", "[{}] {:?}", num, msg);

                    let Some(dkind) = get_dkind(&msg) else {
                        log::error!("Invalid data: {msg:?}");
                        continue;
                    };

                    let msg = vec![msg.to_string()];
                    let payload = MsgPayload::new("CWB", &dkind, msg);
                    if let Err(e) = sender.send(Arc::new(payload)) {
                        log::error!("{e}");
                    }
                }
                Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                Err(e) => log::error!("{e}"),
            };
            // println!("sleep...");
            // sleep(Duration::from_millis(50));
        }
    });

    Ok(handle)
}
