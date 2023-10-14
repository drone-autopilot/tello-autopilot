use std::{net::{IpAddr, Ipv4Addr}, env, time::Duration};
use log::{info, error};
use tello_autopilot::{state::State, cmd::Command};
use tokio::{net::{UdpSocket, TcpStream, TcpListener}, time::{self, timeout}, io::{AsyncWriteExt, AsyncReadExt}};
use std::str;

const LOCAL_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(0,0,0,0));
const TELLO_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(192,168,10,1));
const WATCHDOG_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(127,0,0,1));

const TELLO_CMD_PORT: u16 = 8889;
const TELLO_STATE_PORT: u16 = 8890;
const TELLO_VIDEO_STREAM_PORT: u16 = 11111;
const TELLO_STREAM_ACCESS_PORT: u16 = 62512;

const SERVER_CMD_PORT: u16 = 8989;
const SERVER_STATE_PORT: u16 = 8990;
const SERVER_VIDEO_STREAM_PORT: u16 = 11112;

const PILOT_VIDEO_STREAM_PORT: u16 = 11113;

const TIMEOUT_MILLS: u64 = 100;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let cmd_socket = UdpSocket::bind((LOCAL_IP, TELLO_CMD_PORT)).await?;
    let state_socket = UdpSocket::bind((LOCAL_IP, TELLO_STATE_PORT)).await?;
    let video_socket = UdpSocket::bind((LOCAL_IP, TELLO_VIDEO_STREAM_PORT)).await?;

    let send_only_socket = UdpSocket::bind("0.0.0.0:0").await?;

    // コマンド関連の監視
    info!("Server: Waiting connection from watchdog...");
    let cmd_listener = TcpListener::bind((LOCAL_IP, SERVER_CMD_PORT)).await?;
    match cmd_listener.accept().await {
        Ok((stream, addr)) => {
            info!("Connected to: {:?}", addr);
            tokio::spawn(async move {
                if let Err(err) = listen_cmd(cmd_socket, stream).await {
                    error!("Error in socket2: {:?}", err);
                }
            });
        },
        Err(err) => {
            error!("Error: {:?}", err);
        },
    }

    // ステータスの受信
    info!("Server: Waiting connection from watchdog...");
    let state_listener = TcpListener::bind((LOCAL_IP, SERVER_STATE_PORT)).await?;
    match state_listener.accept().await {
        Ok((stream, addr)) => {
            info!("Connected to: {:?}", addr);
            tokio::spawn(async move {
                if let Err(err) = listen_state(state_socket, stream).await {
                        error!("Error in socket1: {:?}", err);
                }
            });
        },
        Err(err) => {
            error!("Error: {:?}", err);
        },
    }

    // ビデオストリームの監視
    tokio::spawn(async move {
        if let Err(err) = listen_video(video_socket).await {
            error!("Error in socket1: {:?}", err);
        }
    });

    // 標準入力の監視
    tokio::spawn(async {
        listen_stdin(send_only_socket).await;
    });

    // メインスレッドが終了しないように待機
    tokio::signal::ctrl_c().await?;

    Ok(())
}

async fn listen_cmd(socket: UdpSocket, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024];
    socket.send_to("".as_bytes(), (TELLO_IP, TELLO_CMD_PORT)).await?;

    socket.send_to("command".as_bytes(), (TELLO_IP, TELLO_CMD_PORT)).await?;
    let sleep_duration = Duration::from_secs(1);
    time::sleep(sleep_duration).await;
    socket.send_to("streamon".as_bytes(), (TELLO_IP, TELLO_CMD_PORT)).await?;

    loop {
        let receive_task1 = socket.recv_from(&mut buf);
        let timeout1 = timeout(Duration::from_millis(TIMEOUT_MILLS), receive_task1);
        match timeout1.await {
            Ok(result) => {
                match result { Ok((size, peer)) => {
                    info!("Received {} bytes from {}: {:?}", size, peer, str::from_utf8(&buf[..size]));
                    let send_task = stream.write_all(&buf[..size]);
                    let _ = timeout(Duration::from_millis(TIMEOUT_MILLS), send_task).await;
                },
                Err(e) => {
                    error!("Error receiving data: {:?}", e);
                },}
            }
            Err(_) => {}
        }

        let receive_task2 = stream.read(&mut buf);
        let timeout2 = timeout(Duration::from_millis(TIMEOUT_MILLS), receive_task2);
        match timeout2.await {
            Ok(result) => {
                match result { Ok(size) => {
                    if let Ok(data) = str::from_utf8(&buf[..size]) {
                        info!("Received {} bytes : {:?}", size, data);
                        if let Some(cmd) = Command::from_str(data) {
                            let _ = socket.send_to(cmd.to_string().as_bytes(), (TELLO_IP, TELLO_CMD_PORT)).await;
                        } else {
                            let send_task = stream.write_all("Invalid command".as_bytes());
                            let _ = timeout(Duration::from_millis(TIMEOUT_MILLS), send_task).await;
                        }
                    }
                },
                Err(_e) => {
                    // error!("Error receiving data: {:?}", e);
                },}
            }
            Err(_) => {}
        }
    }
}

async fn listen_state(socket: UdpSocket, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024];
    socket.send_to("".as_bytes(), (TELLO_IP, TELLO_CMD_PORT)).await?;
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, _peer)) => {
                // info!("Received {} bytes from {}: {:?}", size, peer, str::from_utf8(&buf[..size]));
                match str::from_utf8(&buf[..size]) {
                    Ok(str) => {
                        let state = State::from_str(str);
                        let _ = stream.write_all(serde_json::to_string(&state).unwrap().as_bytes()).await;
                    },
                    Err(_) => todo!(),
                }
            },
            Err(e) => {
                error!("Error receiving data: {:?}", e);
            },
        }
    }
}

async fn listen_video(socket: UdpSocket) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1460];
    socket.send_to("".as_bytes(), (TELLO_IP, TELLO_STREAM_ACCESS_PORT)).await?;
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, _peer)) => {
                // info!("Received {} bytes from {}: {:?}", size, peer, &buf[..size]);
                let _ = socket.send_to(&buf[..size], (WATCHDOG_IP, SERVER_VIDEO_STREAM_PORT)).await;
                // 追加送信(自動制御用)
                let _ = socket.send_to(&buf[..size], (WATCHDOG_IP, PILOT_VIDEO_STREAM_PORT)).await;
            },
            Err(_e) => {
                // error!("Error receiving data: {:?}", e);
            },
        }
    }
}

async fn listen_stdin(socket: UdpSocket) {
    let stdin = async_std::io::stdin();
    let mut line = String::new();

    loop {
        if let Ok(_) = stdin.read_line(&mut line).await {
            let str_cmd = line.trim();
            info!("Console typed: {}", str_cmd);
            if let Some(cmd) = Command::from_str(str_cmd) {
                let cmd_str = cmd.to_string();
                let _ = socket.send_to(cmd_str.as_bytes(), (TELLO_IP, TELLO_CMD_PORT));
            } else {
                error!("Invalid command.");
            }
            line.clear();
        }
    }
}
