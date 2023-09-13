use std::thread;
use std::net::{IpAddr, UdpSocket};
use std::time::Duration;

const TELLO_CMD_PORT: u16 = 8889;
const TELLO_STATE_PORT: u16 = 8890;

pub struct Tello {
    timeout_dur: Duration,
    local_ip: IpAddr,
    tello_ip: IpAddr,
}

impl Tello {
    pub fn new(timeout_secs: u64, local_ip: IpAddr, tello_ip: IpAddr) -> Self {
        Self {
            timeout_dur: Duration::from_secs(timeout_secs),
            local_ip,
            tello_ip,
        }
    }

    pub fn listen_state(&self) {
        let addr = (self.local_ip, TELLO_STATE_PORT);
        thread::spawn(move || {
            let socket = UdpSocket::bind(addr).expect("Failed to bind to socket");
            loop {
                let mut buffer = [0u8; 1024];

                match socket.recv_from(&mut buffer) {
                    Ok((size, source)) => {
                        // 受信したデータを表示
                        let data = &buffer[..size];
                        let data_str = String::from_utf8_lossy(data);
                        println!("Received {} bytes from {}: {}", size, source, data_str);
                    }
                    Err(err) => {
                        eprintln!("Error receiving data: {}", err);
                    }
                }
            }
        });
    }

    pub fn send_cmd(&self, cmd: &str, wait: bool) -> bool {
        if let Ok(sock) = UdpSocket::bind((self.local_ip, 0)) {
            if let Err(err) = sock.set_broadcast(true) {
                eprintln!("Failed to set broadcast: {}", err);
                return false;
            }

            if let Err(err) = sock.set_read_timeout(Some(self.timeout_dur)) {
                eprintln!("Failed to set read timeout: {}", err);
                return false;
            }

            if let Err(err) = sock.send_to(cmd.as_bytes(), (self.tello_ip, TELLO_CMD_PORT)) {
                eprintln!("Failed to send command: {}", err);
                return false;
            }

            if wait {
                println!("Sending command: {} to {}", cmd, &self.tello_ip);
                let mut buff = [0; 1024];
                if let Ok((recv_size, src)) = sock.recv_from(&mut buff) {
                    if let Ok(v) = String::from_utf8(buff[..recv_size].to_vec()) {
                        println!("From {}: {}", src, v);
                        return true;
                    } else {
                        eprintln!("Failed to convert to string from u8 array");
                    }
                } else {
                    eprintln!("Failed to receive message");
                }

                return false;
            } else {
                println!("Command sent: {} to {}", cmd, &self.tello_ip);
                return true;
            }
        } else {
            eprintln!("Failed to start sender");
            return false;
        }
    }
}