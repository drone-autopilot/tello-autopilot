use std::{
    env,
    io::{self, BufRead},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use log::{error, info};
use tello::{
    cmd::{Command, CommandResult},
    Tello,
};
use tello_autopilot::watchdog::WatchdogServer;

mod tello;

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let stdin = io::stdin();

    // watchdog server
    let arc_watchdog_server = match WatchdogServer::new(300, "127.0.0.1".parse().unwrap()) {
        Ok(ws) => Arc::new(Mutex::new(ws)),
        Err(err) => {
            error!("Watchdog server: {:?}", err);
            return;
        }
    };

    info!("Watchdog server: Waiting connection from client...");
    if let Err(err) = arc_watchdog_server.lock().unwrap().wait_for_connection() {
        error!("Watchdog server: {:?}", err);
        return;
    }

    // command thread
    let ws_clone0 = Arc::clone(&arc_watchdog_server);
    let t0 = thread::spawn(move || loop {
        if let Ok(ref mut mutex) = ws_clone0.try_lock() {
            println!("Watchdog server: {:?}", mutex.receive_cmd());
        }

        thread::sleep(Duration::from_millis(100));
    });

    // tello
    let arc_tello = match Tello::new(
        300,
        "0.0.0.0".parse().unwrap(),
        "192.168.10.1".parse().unwrap(),
    ) {
        Ok(tello) => Arc::new(Mutex::new(tello)),
        Err(err) => {
            error!("Tello: {:?}", err);
            return;
        }
    };

    if let Err(err) = arc_tello.lock().unwrap().connect() {
        error!("Tello: {:?}", err);
        return;
    }

    match arc_tello.lock().unwrap().send_cmd(Command::Command, true) {
        Ok(res) => {
            if res.unwrap() != CommandResult::Ok {
                error!("Tello: Invalid command result");
                return;
            }
        }
        Err(err) => {
            error!("Tello: {:?}", err);
            return;
        }
    }

    match arc_tello.lock().unwrap().send_cmd(Command::StreamOn, true) {
        Ok(res) => {
            if res.unwrap() != CommandResult::Ok {
                error!("Tello: Failed to enable video stream");
                return;
            }
        }
        Err(err) => {
            error!("Tello: {:?}", err);
            return;
        }
    }

    match arc_tello.lock().unwrap().send_cmd(Command::StreamOn, true) {
        Ok(res) => {
            if res.unwrap() != CommandResult::Ok {
                error!("Tello: Failed to enable video stream");
                return;
            }
        }
        Err(err) => {
            error!("Tello: {:?}", err);
            return;
        }
    }

    // state thread
    let tello_clone0 = Arc::clone(&arc_tello);
    let t3 = thread::spawn(move || loop {
        if let Ok(ref mut mutex) = tello_clone0.try_lock() {
            println!("Tello: {:?}", mutex.receive_state());
        }

        thread::sleep(Duration::from_millis(100));
    });

    //video stream thread
    let tello_clone1 = Arc::clone(&arc_tello);
    let t4 = thread::spawn(move || {
        let mut buf = [0; 1460];

        loop {
            if let Ok(ref mut mutex) = tello_clone1.try_lock() {
                println!("{:?}", mutex.receive_video_stream(&mut buf));
            }

            thread::sleep(Duration::from_millis(400));
        }
    });

    // command thread
    let tello_clone2 = Arc::clone(&arc_tello);
    let t5 = thread::spawn(move || {
        let mut input = String::new();

        loop {
            stdin
                .lock()
                .read_line(&mut input)
                .expect("Failed to read line from stdin");

            if let Some(cmd) = Command::from_str(input.trim()) {
                if let Ok(ref mut mutex) = tello_clone2.try_lock() {
                    println!("{:?}", mutex.send_cmd(cmd, true));
                } else {
                    error!("Tello: Instance was locked, Please retry to send command");
                }
            } else {
                error!("Tello: Invalid command: \"{}\"", input.trim());
            }

            input.clear();
        }
    });

    t0.join().unwrap();
    t3.join().unwrap();
    t4.join().unwrap();
    t5.join().unwrap();
}
