mod component;
mod config;

use component::{reader_serial_port::setup_serial_port_cwb, receiver_raw::setup_rawdata_recorder, receiver_sqlite::setup_sqlite3_recorder};
use config::SystemConfig;
use std::{process::exit, sync::mpsc};

fn main() {
    let config = SystemConfig::load("config/config.json.ignore").expect("load config failed");
    log4rs::init_file(&config.global.log4rs_cfg, Default::default()).unwrap();
    
    log::info!(target: "configuation", "{config:?}");

    let (uart_tx, uart_rx) = mpsc::channel();
    let (rc_raw_tx, rc_raw_rx) = mpsc::channel();
    let (rc_sqlite_tx, rc_sqlite_rx) = mpsc::channel();

    let Ok(_uart_handler) = setup_serial_port_cwb(
        &config.global.serial_port.path,
        config.global.serial_port.baudrate,
        uart_tx,
    ) else {
        log::error!("Setup serial port failed");
        exit(exitcode::UNAVAILABLE);
    };

    let Ok(_rc_raw_handler) = setup_rawdata_recorder(rc_raw_rx, &config.global.database.rawdata)
    else {
        log::error!("Setup raw data recorder failed");
        exit(exitcode::UNAVAILABLE);
    };

    let Ok(_rc_sqlite_handler) = setup_sqlite3_recorder(rc_sqlite_rx, &config.global.database.sqlite3)
    else {
        log::error!("Setup raw sqlite3 recorder failed");
        exit(exitcode::UNAVAILABLE);
    };

    // dispatcher
    while let Ok(msg) = uart_rx.recv() {
        // println!("Received and send: {msg}");
        let _ = rc_raw_tx.send(msg.clone());
        let _ = rc_sqlite_tx.send(msg.clone());

        // let MN = CWBMinData::parse_from_str(&msg);
        // println!("{:?}", MN);
    }
}
