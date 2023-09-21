use log::{error, info};
use std::io::{Error, Write, Read};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::{
    net::{IpAddr, UdpSocket},
    str,
};

use crate::tello::cmd::CommandResult;

use self::cmd::Command;

pub mod cmd;
pub mod state;

const TELLO_CMD_PORT: u16 = 8889;
const TELLO_STATE_PORT: u16 = 8890;
const TELLO_STREAM_PORT: u16 = 11111;
const SERVER_CMD_PORT: u16 = 8989;
const SERVER_STATE_PORT: u16 = 8990;
const SERVER_STREAM_PORT: u16 = 11112;

pub struct Tello {
    timeout_dur: Duration,
    local_ip: IpAddr,
    tello_ip: IpAddr,
    watchdog_ip: IpAddr,
}

impl Tello {
    pub fn new(timeout_millis: u64, local_ip: IpAddr, tello_ip: IpAddr, watchdog_ip: IpAddr) -> Self {
        Self {
            timeout_dur: Duration::from_millis(timeout_millis),
            local_ip,
            tello_ip,
            watchdog_ip,
        }
    }

    pub fn listen_state(&self) {
        let local_ip = self.local_ip;
        let timeout_dur = self.timeout_dur;
        let addr = (local_ip, TELLO_STATE_PORT);
        let watchdog_ip = self.watchdog_ip;
        let mut watchdog;

        // ウォッチドッグ: state取得ポートに接続
        match TcpListener::bind((watchdog_ip, SERVER_STATE_PORT)) {
            Ok(listener) => {
                info!("Waiting for a watchdog(state) to connect...");

                match listener.accept() {
                    Ok((stream, _)) => {
                        info!("Watchdog(state) connected!");
                        watchdog = stream;
                    }
                    Err(e) => {
                        error!("Error accepting connection: {:?}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                error!("Failed to bind: {:?}", e);
                return;
            }
        }

        thread::spawn(move || {
            let socket = UdpSocket::bind(addr).expect("Failed to bind to socket");

            if let Err(err) = socket.set_broadcast(true) {
                error!("Failed to set broadcast: {}", err);
            }

            if let Err(err) = socket.set_read_timeout(Some(timeout_dur)) {
                error!("Failed to set read timeout: {}", err);
            }

            let mut buf = [0; 1024];

            loop {
                // info!("Waiting receive...");
                match socket.recv_from(&mut buf) {
                    Ok((size, _)) => match str::from_utf8(&buf[..size]) {
                        Ok(s) => {
                            match CommandResult::from_str(s) {
                                CommandResult::State(state) => {
                                    if let Err(e) = watchdog.write_all(serde_json::to_string(&state).unwrap().as_str().trim().as_bytes()) {
                                        error!("Failed to send message: {:?}", e);
                                    }
                                }
                                _ => (),
                            };
                        }
                        Err(err) => error!("{:?}", err),
                    },
                    Err(err) => {
                        error!("Failed to receive data: {:?}", err);
                    }
                }

                // sleep(Duration::from_secs(1));

            }
        });
    }

    fn listen_stream(&self) {
        let local_ip = self.local_ip;
        let watchdog_ip = self.watchdog_ip;
        let timeout_dur = self.timeout_dur;
        let addr = (local_ip, TELLO_STREAM_PORT);
        thread::spawn(move || {
            let socket = UdpSocket::bind(addr).expect("Failed to bind to socket");

            if let Err(err) = socket.set_broadcast(true) {
                error!("Failed to set broadcast: {}", err);
            }

            if let Err(err) = socket.set_read_timeout(Some(timeout_dur)) {
                error!("Failed to set read timeout: {}", err);
            }

            let mut buf = [0; 1460];

            loop {
                // info!("Waiting receive...");
                match socket.recv_from(&mut buf) {
                    Ok((size, _)) => {
                        let _res =
                            socket.send_to(&buf[..size], (watchdog_ip, SERVER_STREAM_PORT));
                        // info!("udp send: {:?}", res);
                    }
                    Err(_err) => {
                        // error!("Failed to receive data: {:?}", err);
                    }
                }

                // sleep(Duration::from_secs(1));
            }
        });
    }

    pub fn send_cmd(&self, cmd: Command, wait: bool) -> Result<(), Error> {
        let socket = match UdpSocket::bind((self.local_ip, TELLO_CMD_PORT)) {
            Ok(s) => s,
            Err(err) => return Err(err),
        };

        if let Err(err) = socket.set_broadcast(true) {
            return Err(err);
        }

        if let Err(err) = socket.set_read_timeout(Some(self.timeout_dur)) {
            return Err(err);
        }

        if let Err(err) =
            socket.send_to(cmd.to_string().as_bytes(), (self.tello_ip, TELLO_CMD_PORT))
        {
            return Err(err);
        }

        info!("Command sent: {} to {}", cmd, &self.tello_ip);

        if !wait {
            return Ok(());
        }

        info!("Waiting receive...");
        let mut buf = [0; 1024];

        match socket.recv_from(&mut buf) {
            Ok((size, _)) => match str::from_utf8(&buf[..size]) {
                Ok(s) => {
                    println!("{:?}", CommandResult::from_str(s));
                    if cmd == Command::StreamOn {
                        self.listen_stream();
                    }
                }
                Err(err) => error!("{:?}", err), // TODO: return err but incorrected type
            },
            Err(err) => return Err(err),
        }

        Ok(())
    }

    pub fn handle_watchdog(&self) {
        let watchdog_ip = self.watchdog_ip;
        let mut watchdog;

        // ウォッチドッグ: cmd/response用ポートに接続
        match TcpListener::bind((watchdog_ip, SERVER_CMD_PORT)) {
            Ok(listener) => {
                info!("Waiting for a watchdog(cmd/res) to connect...");

                match listener.accept() {
                    Ok((stream, _)) => {
                        info!("Watchdog(cmd/res) connected!");
                        watchdog = stream;
                    }
                    Err(e) => {
                        error!("Error accepting connection: {:?}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                error!("Failed to bind: {:?}", e);
                return;
            }
        }

        thread::spawn(move || {
            let mut buffer = [0; 1024];

            loop {
                match watchdog.read(&mut buffer) {
                    Ok(bytes_read) => {
                        let message = String::from_utf8_lossy(&buffer[0..bytes_read]);
                        println!("Received: {}", message);

                        if let Some(_cmd) = Command::from_str(message.trim()) {
                            //
                        } else {
                            let response = format!("Invalid command: \"{}\"", message.trim());
                            if let Err(e) = watchdog.write_all(response.as_bytes()) {
                                error!("Failed to send message: {:?}", e);
                            }
                        }

                        // send_cmdを利用したい
                        // if let Some(cmd) = Command::from_str(message.trim()) {
                        //     if let Err(err) = self.send_cmd(cmd, true) {
                        //         let response = format!("Error occured in send_cmd: {:?}", err);
                        //         if let Err(e) = watchdog.write_all(response.as_bytes()) {
                        //             error!("Failed to send message: {:?}", e);
                        //         }
                        //     }
                        // } else {
                        //     let response = format!("Invalid command: \"{}\"", message.trim());
                        //     if let Err(e) = watchdog.write_all(response.as_bytes()) {
                        //         error!("Failed to send message: {:?}", e);
                        //     }
                        // }
                    }
                    Err(e) => {
                        error!("Failed to read from watchdog: {:?}", e);
                    }
                }
            }
        });
    }
}
