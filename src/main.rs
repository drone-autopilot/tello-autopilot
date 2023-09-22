use std::{net::{IpAddr, Ipv4Addr}, env, time::Duration, io::{Error, ErrorKind}};
use log::{info, error};
use tello_autopilot::state::State;
use tokio::{net::{UdpSocket, TcpStream, TcpListener}, time::{self}, io::AsyncWriteExt};
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let cmd_socket = UdpSocket::bind((LOCAL_IP, TELLO_CMD_PORT)).await?;
    let state_socket = UdpSocket::bind((LOCAL_IP, TELLO_STATE_PORT)).await?;
    let video_socket = UdpSocket::bind((LOCAL_IP, TELLO_VIDEO_STREAM_PORT)).await?;

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

    tokio::spawn(async move {
        if let Err(err) = listen_video(video_socket).await {
            error!("Error in socket1: {:?}", err);
        }
    });

    // メインスレッドが終了しないように待機
    tokio::signal::ctrl_c().await?;

    Ok(())
}

async fn listen_cmd(socket: UdpSocket, mut stream: TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    let mut buf = vec![0; 1024];
    //socket.set_broadcast(true)?;
    socket.send_to("command".as_bytes(), (TELLO_IP, TELLO_CMD_PORT)).await?;
    let sleep_duration = Duration::from_secs(1);
    time::sleep(sleep_duration).await;
    socket.send_to("streamon".as_bytes(), (TELLO_IP, TELLO_CMD_PORT)).await?;
    loop {
        match socket.recv_from(&mut buf).await {
            Ok((size, peer)) => {
                info!("Received {} bytes from {}: {:?}", size, peer, str::from_utf8(&buf[..size]));
                let _ = stream.write_all(&buf[..size]).await;
            },
            Err(e) => {
                error!("Error receiving data: {:?}", e);
            },
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
                socket.send_to(&buf[..size], (WATCHDOG_IP, SERVER_VIDEO_STREAM_PORT)).await?;
            },
            Err(_e) => {
                // error!("Error receiving data: {:?}", e);
            },
        }
    }
}
