mod component;
mod config;

use component::{
    reader_loggernet::setup_file_listen_naho, reader_serial_port::setup_serial_port_cwb,
    receiver_raw::setup_rawdata_recorder, receiver_sqlite::setup_sqlite3_recorder,
};
use config::SystemConfig;
use std::{
    process::exit,
    sync::{mpsc, Arc},
};

fn main() {
    let config = Arc::new(SystemConfig::load("config/config.json").expect("load config failed"));
    log4rs::init_file(&config.global.log4rs_cfg, Default::default()).unwrap();

    log::info!(target: "configuation", "{config:?}");

    // (name, handler)
    let mut handlers = Vec::new();

    let (uart_tx, uart_rx) = mpsc::channel();
    let (rc_raw_tx, rc_raw_rx) = mpsc::channel();
    let (rc_sqlite_tx, rc_sqlite_rx) = mpsc::channel();

    // let Ok(_uart_handler) = setup_serial_port_cwb_by_line(
    if let Ok(handler) = setup_serial_port_cwb(
        &config.global.serial_port.path,
        config.global.serial_port.baudrate,
        uart_tx.clone(),
    ) {
        log::info!("Setup serial port success.");
        log::info!(target: "info", "Setup serial port success.");
        handlers.push(("serialport", handler));
    } else {
        log::error!("Setup serial port failed.");
        exit(exitcode::UNAVAILABLE);
    };

    if let Ok(handler) = setup_file_listen_naho(config.clone(), uart_tx.clone()) {
        log::info!("Setup loggernet listener success.");
        log::info!(target: "info", "Setup loggernet listener success.");
        handlers.push(("loggernet", handler));
    } else {
        log::error!("Setup loggernet listener failed.");
    }

    if let Ok(handler) = setup_rawdata_recorder(rc_raw_rx, config.clone()) {
        log::info!("Setup rawdata recorder success.");
        log::info!(target: "info", "Setup rawdata recorder success.");
        handlers.push(("rawdata", handler));
    } else {
        log::error!("Setup rawdata recorder failed.");
        exit(exitcode::UNAVAILABLE);
    };

    if let Ok(handler) = setup_sqlite3_recorder(rc_sqlite_rx, config.clone()) {
        log::info!("Setup sqlite3 recorder success.");
        log::info!(target: "info", "Setup sqlite3 recorder success.");
        handlers.push(("sqlite3", handler));
    } else {
        log::error!("Setup sqlite3 recorder failed.");
        exit(exitcode::UNAVAILABLE);
    };

    drop(uart_tx); // release last unused tx

    // dispatcher
    while let Ok(msg) = uart_rx.recv() {
        let _ = rc_raw_tx.send(msg.clone());
        let _ = rc_sqlite_tx.send(msg.clone());
    }
}
