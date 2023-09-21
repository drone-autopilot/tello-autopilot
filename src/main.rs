use std::{
    env,
    io::{self, BufRead},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use log::{error, info};
use tello_autopilot::{
    tello::{
        cmd::{Command, CommandResult},
        Tello,
    },
    watchdog::WatchdogServer,
};

fn main() {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let stdin = io::stdin();

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
    let tello_clone0 = Arc::clone(&arc_tello);
    let ws_clone0 = Arc::clone(&arc_watchdog_server);
    let t0 = thread::spawn(move || loop {
        let mut c = None;

        if let Ok(ref mut mutex) = ws_clone0.try_lock() {
            println!("Watchdog server: {:?}", mutex.receive_cmd());
            if let Ok(cmd) = mutex.receive_cmd() {
                c = Command::from_str(cmd.as_str());
            }
        }

        if let Some(cmd) = c {
            if let Ok(ref mut mutex) = tello_clone0.try_lock() {
                println!("{:?}", mutex.send_cmd(cmd, true));
            } else {
                error!("Tello: Instance was locked, Please retry to send command");
            }
        }

        thread::sleep(Duration::from_millis(100));
    });

    // state thread
    let tello_clone1 = Arc::clone(&arc_tello);
    let ws_clone1 = Arc::clone(&arc_watchdog_server);
    let t3 = thread::spawn(move || loop {
        let mut s = None;
        if let Ok(ref mut m_tello) = tello_clone1.try_lock() {
            if let Ok(state) = m_tello.receive_state() {
                s = state;
            }
        }

        if let Some(state) = s {
            if let Ok(ref mut m_ws) = ws_clone1.try_lock() {
                m_ws.send_tello_state(state);
            }
        }

        thread::sleep(Duration::from_millis(100));
    });

    // video stream thread
    let tello_clone2 = Arc::clone(&arc_tello);
    //let ws_clone2 = Arc::clone(&arc_watchdog_server);
    let t4 = thread::spawn(move || {
        let mut buf = [0; 1460];

        loop {
            if let Ok(ref mut m_tello) = tello_clone2.try_lock() {
                m_tello.receive_and_send_video_stream(&mut buf);
            }

            // if let Ok(ref mut m_ws) = ws_clone2.try_lock() {
            //     m_ws.send_video_stream(&buf);
            // }
        }
    });

    // command thread
    let tello_clone3 = Arc::clone(&arc_tello);
    let t5 = thread::spawn(move || {
        let mut input = String::new();

        loop {
            stdin
                .lock()
                .read_line(&mut input)
                .expect("Failed to read line from stdin");

            if let Some(cmd) = Command::from_str(input.trim()) {
                if let Ok(ref mut mutex) = tello_clone3.try_lock() {
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
