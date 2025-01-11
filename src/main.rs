mod component;
mod config;

use component::{parser_cwb::CWBMinData, reader_serial_port::setup_serial_port_cwb, receiver_raw::setup_rawdata_recorder};
use config::SystemConfig;
use log::{debug, error, info, trace, warn};
use std::{
    io::{self, BufRead, BufReader},
    sync::mpsc,
    thread::sleep,
    time::Duration,
};

fn main() {
    println!("Hello, world!");
    let config = SystemConfig::load("config/config.json").expect("load config failed");
    log4rs::init_file(&config.global.log4rs_cfg, Default::default()).unwrap();

    println!("{config:#?}");

    let (uart_tx, uart_rx) = mpsc::channel();
    let (rc_raw_tx, rc_raw_rx) = mpsc::channel();

    let _uart_handler = setup_serial_port_cwb(
        &config.global.serial_port.path,
        config.global.serial_port.baudrate,
        uart_tx,
    )
    .expect("Setup serial port failed");

    let _rc_raw_handler = setup_rawdata_recorder(rc_raw_rx, &config.global.database.rawdata)
        .expect("Setup raw data recorder failed");

    // dispatcher
    while let Ok(msg) = uart_rx.recv() {
        // println!("Received and send: {msg}");
        let _ = rc_raw_tx.send(msg.clone());

        // let MN = CWBMinData::parse_from_str(&msg);
        // println!("{:?}", MN);
    }
}
