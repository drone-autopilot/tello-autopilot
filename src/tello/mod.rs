use log::{error, info};
use std::io::Error;
use std::thread;
// use std::thread::sleep;
use std::time::Duration;
use std::{
    net::{IpAddr, UdpSocket},
    str,
};

use crate::server::Server;

use self::cmd::Command;

pub mod cmd;
pub mod state;

const TELLO_CMD_PORT: u16 = 8889;
const TELLO_STATE_PORT: u16 = 8890;

pub struct Tello {
    timeout_dur: Duration,
    local_ip: IpAddr,
    tello_ip: IpAddr,
}

impl Tello {
    pub fn new(timeout_millis: u64, local_ip: IpAddr, tello_ip: IpAddr) -> Self {
        Self {
            timeout_dur: Duration::from_millis(timeout_millis),
            local_ip,
            tello_ip,
        }
    }

    pub fn listen_state(&self, mut server: Server) {
        let local_ip = self.local_ip;
        let timeout_dur = self.timeout_dur;
        let addr = (local_ip, TELLO_STATE_PORT);
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
                    Ok((size, _s_addr)) => {
                        // info!(
                        //     "Received {} bytes from {}: {:?}",
                        //     size,
                        //     s_addr,
                        //     str::from_utf8(&buf[..size])
                        // );
                        if let Ok(data) = str::from_utf8(&buf[..size]){
                            server.send_message(data);
                        }
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
        let (size, s_addr) = match socket.recv_from(&mut buf) {
            Ok((size, s_addr)) => (size, s_addr),
            Err(err) => return Err(err),
        };

        info!(
            "Received {} bytes from {}: {:?}",
            size,
            s_addr,
            str::from_utf8(&buf[..size])
        );

        Ok(())
    }
}
