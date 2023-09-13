use std::error::Error;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::str;
use std::time::Duration;
use tokio::net::UdpSocket as Tokio;

const TELLO_CMD_PORT: u16 = 8889;
const LOCAL_STATE_PORT: u16 = 8890;
const LOCAL_VIDEO_STREAM_PORT: u16 = 11111;

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

    pub async fn listen_state(&self) -> Result<(), Box<dyn Error>> {
        let addr = SocketAddr::new(self.local_ip, LOCAL_STATE_PORT);
        let socket = Tokio::bind(addr).await?;

        let mut buff = [0; 1024];
        loop {
            match socket.recv_from(&mut buff).await {
                Ok((recv_size, _sec)) => match String::from_utf8(buff[..recv_size].to_vec()) {
                    Ok(v) => println!("{}", v),
                    Err(v) => println!("failed to convert to string from u8 array: {}", v),
                },
                Err(_) => println!("failed to receive message"),
            }
        }
    }

    pub fn send_cmd(&self, cmd: &str, wait: bool) -> bool {
        if let Ok(sock) = UdpSocket::bind(SocketAddr::new(self.local_ip, TELLO_CMD_PORT)) {
            if let Err(err) = sock.set_broadcast(true) {
                eprintln!("Failed to set broadcast: {}", err);
                drop(sock);
                return false;
            }

            if let Err(err) = sock.set_read_timeout(Some(self.timeout_dur)) {
                eprintln!("Failed to set read timeout: {}", err);
                drop(sock);
                return false;
            }

            if let Err(err) = sock.send_to(
                cmd.as_bytes(),
                SocketAddr::new(self.tello_ip, TELLO_CMD_PORT),
            ) {
                eprintln!("Failed to send command: {}", err);
                drop(sock);
                return false;
            }

            if wait {
                println!("Sending command: {} to {}", cmd, &self.tello_ip);
                let mut buff = [0; 1024];
                if let Ok((recv_size, src)) = sock.recv_from(&mut buff) {
                    if let Ok(v) = String::from_utf8(buff[..recv_size].to_vec()) {
                        println!("From {}: {}", src, v);
                        drop(sock);
                        return true;
                    } else {
                        eprintln!("Failed to convert to string from u8 array");
                    }
                } else {
                    eprintln!("Failed to receive message");
                }

                drop(sock);
                return false;
            } else {
                println!("Command sent: {} to {}", cmd, &self.tello_ip);
                drop(sock);
                return true;
            }
        } else {
            eprintln!("Failed to start sender");
            return false;
        }
    }
}
