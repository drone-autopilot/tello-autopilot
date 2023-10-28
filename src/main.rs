use log::{error, info};
use std::{env, io::ErrorKind, time::Duration};
use tello_autopilot::{cmd::Command, state::State};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
};

// const LOCAL_IP: &'static str = "0.0.0.0";
// const TELLO_IP: &'static str = "192.168.10.1";

// const TELLO_CMD_PORT: u16 = 8889;
// const TELLO_STATE_PORT: u16 = 8890;
// const TELLO_VIDEO_STREAM_PORT: u16 = 11111;
// const TELLO_STREAM_ACCESS_PORT: u16 = 62512;

// const SERVER_CMD_PORT: u16 = 8989;
// const SERVER_STATE_PORT: u16 = 8990;
const LISTEN_CMD_ADDR: (&'static str, u16) = ("127.0.0.1", 8989);
const LISTEN_STATE_ADDR: (&'static str, u16) = ("127.0.0.1", 8990);

const TIMEOUT_MS: u64 = 15000; // 15s

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async fn sleep_3s() {
        tokio::time::sleep(tokio::time::Duration::from_millis(3000)).await;
    }

    // TODO: not working
    fn send_cmd(cmd: Command) {
        tokio::spawn(async move {
            if let Err(e) = shoot_cmd(LISTEN_CMD_ADDR, &cmd).await {
                error!("sned cmd: {:?}", e);
            }
        });
    }

    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // command
    tokio::spawn(async move {
        if let Err(e) = listen_and_send_cmd(LISTEN_CMD_ADDR, ("192.168.10.1", 8889)).await {
            error!("listen cmd: {:?}", e);
        }
    });
    sleep_3s().await;

    tokio::spawn(async move {
        if let Err(e) = listen_stdin(LISTEN_CMD_ADDR).await {
            error!("listen stdin: {:?}", e);
        }
    });

    //send_cmd(Command::Command);

    tokio::spawn(async move {
        if let Err(e) = shoot_cmd_infinitely(LISTEN_CMD_ADDR, &Command::Command, 15000).await {
            error!("listen shoot cmd: {:?}", e);
        }
    });

    sleep_3s().await;

    tokio::spawn(async move {
        if let Err(e) = listen_and_send_state(LISTEN_STATE_ADDR, ("0.0.0.0", 8890)).await {
            error!("Error in listen state thread: {:?}", e);
        }
    });

    // video
    //send_cmd(Command::StreamOn);

    // tokio::spawn(async move {
    //     if let Err(e) = listen_and_stream_video(
    //         ("0.0.0.0", 11111),
    //         ("192.168.10.1", 62512),
    //         &[
    //             ("127.0.0.1", 11112), // watchdog
    //             ("127.0.0.1", 11113), // detector
    //         ],
    //     )
    //     .await
    //     {
    //         error!("listen video: {:?}", e);
    //     }
    // });

    tokio::signal::ctrl_c().await?;

    Ok(())
}

async fn listen_and_send_cmd<A: tokio::net::ToSocketAddrs + Copy + Send + 'static>(
    listen_target: A,
    dst_target: A,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(listen_target).await?;

    // multi clients
    loop {
        info!("listen cmd: Waiting connection...");
        let (mut stream, addr) = match listener.accept().await {
            Ok(r) => r,
            Err(e) => return Err(Box::new(e)),
        };

        let dst_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        let mut buf = vec![0; 1024];
        info!("listen cmd: Connected from {}", addr);

        tokio::spawn(async move {
            loop {
                let size = match stream.read(&mut buf).await {
                    Ok(size) => size,
                    Err(e) => {
                        error!(
                            "listen cmd: Error while reading from client ({}): {:?}",
                            addr, e
                        );
                        break;
                    }
                };

                if size == 0 {
                    continue;
                }

                let data = &buf[..size];
                let s = String::from_utf8_lossy(data);

                let cmd = match Command::from_str(&s) {
                    Some(cmd) => cmd,
                    None => {
                        error!("Invalid command: \"{}\"", s);
                        stream.write_all("error".as_bytes()).await.unwrap();
                        buf.fill(0);
                        continue;
                    }
                };

                info!(
                    "listen cmd: Receive command from client ({}): {:?}",
                    addr, cmd
                );

                if let Err(e) = dst_socket
                    .send_to(cmd.to_string().as_bytes(), dst_target)
                    .await
                {
                    error!("listen cmd: Failed to send cmd to target: {:?}", e);
                    stream.write_all("error".as_bytes()).await.unwrap();
                    buf.fill(0);
                    continue;
                }

                // wait response
                let size = match dst_socket.recv_from(&mut buf).await {
                    Ok((size, _)) => size,
                    Err(e) => {
                        error!(
                            "listen cmd: Failed to receive response from target: {:?}",
                            e
                        );
                        stream.write_all("error".as_bytes()).await.unwrap();
                        buf.fill(0);
                        continue;
                    }
                };

                let s = String::from_utf8_lossy(&buf[..size]);
                info!("listen cmd: Receive response from target: {:?}", s);
                stream.write_all(s.as_bytes()).await.unwrap();

                buf.fill(0);
            }
            info!("listen cmd: End of connection with client ({})", addr);
        });
    }
}

async fn listen_stdin<A: tokio::net::ToSocketAddrs + Copy>(
    target: A,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = async_std::io::stdin();
    let mut line = String::new();

    let mut stream = TcpStream::connect(target).await?;

    loop {
        if stdin.read_line(&mut line).await.is_err() {
            continue;
        }

        let s = line.trim();
        let cmd = match Command::from_str(s) {
            Some(cmd) => cmd,
            None => {
                error!("Invalid command: \"{}\"", s);
                line.clear();
                continue;
            }
        };

        if let Err(e) = stream.write_all(cmd.to_string().as_bytes()).await {
            error!("{:?}", e);
        }

        line.clear();
    }
}

async fn shoot_cmd<A: tokio::net::ToSocketAddrs>(
    target: A,
    cmd: &Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(target).await?;
    stream.write_all(cmd.to_string().as_bytes()).await?;

    Ok(())
}

async fn shoot_cmd_infinitely<A: tokio::net::ToSocketAddrs + Copy>(
    target: A,
    cmd: &Command,
    dur_ms: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(target).await?;
    let s = cmd.to_string();

    loop {
        stream.write_all(s.as_bytes()).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(dur_ms as u64)).await;
    }
}

async fn listen_and_send_state<A: tokio::net::ToSocketAddrs + Copy + Send + 'static>(
    listen_target: A,
    src_target: A,
) -> Result<(), Box<dyn std::error::Error>> {
    //let listener = TcpListener::bind(listen_target).await?;
    let src_socket = UdpSocket::bind(src_target).await?;
    let mut buf = vec![0; 1024];

    loop {
        let size = match src_socket.recv_from(&mut buf).await {
            Ok((size, _)) => size,
            Err(e) => {
                error!(
                    "listen state: Failed to receive response from target: {:?}",
                    e
                );
                continue;
            }
        };

        let s = String::from_utf8_lossy(&buf[..size]).to_string();
        let state = State::from_str(s.as_str());
        info!("listen state: Receive state from target: {:?}", state);
        buf.fill(0);
    }
}

async fn listen_and_stream_video<A: tokio::net::ToSocketAddrs>(
    listen_target: A,
    src_target: A,
    dst_target: &[A],
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind(listen_target).await?;
    let mut buf = vec![0; 1460];
    socket.send_to("".as_bytes(), src_target).await?;

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, _)) => {
                for target in dst_target {
                    // ignore errors
                    let _ = socket.send_to(&buf[..size], target).await;
                }
            }
            Err(_e) => {
                //error!("Error while listening video: {:?}", e);
            }
        }
    }
}
