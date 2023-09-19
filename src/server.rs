use log::{error, info};
use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::thread;

pub struct Server {
    ip: IpAddr,
    port: u16,
    client: Option<TcpStream>,
}

impl Server {
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self {
            ip,
            port,
            client: None,
        }
    }

    pub fn connect(&mut self) {
        if self.client.is_some() {
            error!("TcpListener has been created.");
            return;
        }

        match TcpListener::bind((self.ip, self.port)) {
            Ok(listener) => {
                info!("Waiting for a watchdog to connect...");

                match listener.accept() {
                    Ok((stream, _)) => {
                        self.handle_client(stream);
                    }
                    Err(e) => {
                        error!("Error accepting connection: {:?}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to bind: {:?}", e);
            }
        }
    }

    fn handle_client(&mut self, mut stream: TcpStream) {
        self.client = Some(stream.try_clone().unwrap());
        info!("Client connected!");

        thread::spawn(move || {
            let mut buffer = [0; 1024];

            while let Ok(bytes_read) = stream.read(&mut buffer) {
                if bytes_read == 0 {
                    break;
                }

                let message = String::from_utf8_lossy(&buffer[0..bytes_read]);
                println!("Received: {}", message);
            }
        });
    }

    pub fn send_message(&mut self, msg: &str) {
        let mut client = match self.client.take() {
            Some(client) => client,
            None => {
                error!("TcpStream not available.");
                return;
            }
        };

        if let Err(e) = client.write_all(msg.as_bytes()) {
            error!("Failed to send message: {:?}", e);
        }

        self.client = Some(client);
    }
}
