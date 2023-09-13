use std::net::UdpSocket;
use std::str;
use std::time::Duration;
use std::error::Error;
use tokio::net::UdpSocket as Tokio;

impl Tello {
    pub fn new(timeout: u64, local_ip: impl Into<String>, tello_ip: impl Into<String>) -> Tello {
        Tello {
            timeout,
            local_ip: local_ip.into(),
            tello_ip: tello_ip.into(),
        }
    }

    pub async fn listen_state(&self) -> Result<(), Box<dyn Error>> {
        let addr = String::new() + &self.local_ip + ":8890";
        let socket = Tokio::bind(addr).await?;

        let mut buff = [0; 1024];
        loop{
            match socket.recv_from(&mut buff).await {
                Ok((recv_size, _sec)) => {
                    match String::from_utf8(buff[..recv_size].to_vec()) {
                        Ok(v) => println!("{}", v),
                        Err(v) => println!("failed to convert to string from u8 array: {}", v),
                    }
                }
                Err(_) => println!("failed to receive message"),
            }
        }
    }

    pub fn send_cmd(&self, cmd: &str, wait: bool) -> bool {
        if let Ok(sock) = UdpSocket::bind(format!("{}:8889", self.local_ip)) {
            if let Err(err) = sock.set_broadcast(true) {
                eprintln!("Failed to set broadcast: {}", err);
                drop(sock);
                return false;
            }
    
            if let Err(err) = sock.set_read_timeout(Some(Duration::from_secs(self.timeout))) {
                eprintln!("Failed to set read timeout: {}", err);
                drop(sock);
                return false;
            }
    
            if let Err(err) = sock.send_to(cmd.as_bytes(), format!("{}:8889", self.tello_ip)) {
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

pub struct Tello {
    pub timeout: u64,
    pub local_ip: String,
    pub tello_ip: String,
}
