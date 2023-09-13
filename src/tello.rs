use std::net::UdpSocket;
use std::str;
use std::thread;
use std::time::Duration;

impl Tello {
    pub fn new(timeout: u64, local_ip: impl Into<String>, tello_ip: impl Into<String>) -> Tello {
        Tello {
            timeout,
            local_ip: local_ip.into(),
            tello_ip: tello_ip.into(),
        }
    }

    //通信は来ているがrecv_fromで取得できていない
    pub fn listen_state(&self) {
        match UdpSocket::bind(String::new() + &self.local_ip + ":8890") {
            Ok(sock) => {
                let _ = sock.set_read_timeout(Some(Duration::from_secs(self.timeout)));
                let mut buff = [0; 1024];
                thread::spawn(move || loop {
                    match sock.recv_from(&mut buff) {
                        Ok((recv_size, _src)) => {
                            match String::from_utf8(buff[..recv_size].to_vec()) {
                                Ok(v) => println!("{}", v),
                                Err(v) => {
                                    println!("failed to convert to string from u8 array:{}", v)
                                }
                            }
                        }
                        Err(_) => println!("failed to receive message"),
                    }
                });
            }
            Err(e) => println!("failed to start listener: {}", e),
        }
    }

    pub fn send_cmd(&self, cmd: &str, wait: bool) -> bool {
        match UdpSocket::bind(String::new() + &self.local_ip + ":8889") {
            Ok(sock) => {
                sock.set_broadcast(true).expect("failed to set broadcast");
                let _ = sock.set_read_timeout(Some(Duration::from_secs(self.timeout)));
                match sock.send_to(cmd.as_bytes(), String::new() + &self.tello_ip + ":8889") {
                    Ok(_v) => {
                        if wait {
                            println!("sending command: {} to {}", cmd, &self.tello_ip);
                            let mut buff = [0; 1024];
                            match sock.recv_from(&mut buff) {
                                Ok((recv_size, src)) => {
                                    match String::from_utf8(buff[..recv_size].to_vec()) {
                                        Ok(v) => {
                                            println!("from {}: {}", src, v);
                                            true
                                        }
                                        Err(v) => {
                                            println!(
                                                "failed to convert to string from u8 array:{}",
                                                v
                                            );
                                            false
                                        }
                                    }
                                }
                                Err(_) => {
                                    println!("failed to receive message");
                                    false
                                }
                            }
                        } else {
                            println!("command sended: {} to {}", cmd, &self.tello_ip);
                            true
                        }
                    }
                    Err(v) => {
                        println!("caught exception : {}", v);
                        false
                    }
                }
            }
            Err(v) => {
                println!("failed to start sender: {}", v);
                false
            }
        }
    }
}

pub struct Tello {
    pub timeout: u64,
    pub local_ip: String,
    pub tello_ip: String,
}
