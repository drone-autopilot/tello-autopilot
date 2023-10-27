use log::{error, info};
use std::{env, sync::Arc, time::Duration};
use tello_autopilot::{cmd::Command, state::State};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
    sync::Mutex,
};

const LOCAL_IP: &'static str = "0.0.0.0";
const TELLO_IP: &'static str = "192.168.10.1";

const TELLO_CMD_PORT: u16 = 8889;
const TELLO_STATE_PORT: u16 = 8890;
const TELLO_VIDEO_STREAM_PORT: u16 = 11111;
const TELLO_STREAM_ACCESS_PORT: u16 = 62512;

const SERVER_CMD_PORT: u16 = 8989;
const SERVER_STATE_PORT: u16 = 8990;

const TIMEOUT_MS: u64 = 500;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async fn sleep() {
        tokio::time::sleep(tokio::time::Duration::from_millis(TIMEOUT_MS as u64)).await;
    }

    async fn send_cmd(cmd: Command) -> Result<(), Box<dyn std::error::Error>> {
        shoot_cmd(("127.0.0.1", SERVER_CMD_PORT), cmd).await
    }

    env::set_var("RUST_LOG", "info");
    env_logger::init();

    tokio::spawn(async move {
        if let Err(e) =
            listen_and_send_cmd((LOCAL_IP, SERVER_CMD_PORT), (TELLO_IP, TELLO_CMD_PORT)).await
        {
            error!("Error in listen command thread: {:?}", e);
        }
    });

    sleep().await;
    send_cmd(Command::Command).await?;
    send_cmd(Command::StreamOn).await?;

    tokio::spawn(async move {
        if let Err(e) = listen_stdin(("127.0.0.1", SERVER_CMD_PORT)).await {
            error!("Error in listen stdin thread: {:?}", e);
        }
    });

    tokio::spawn(async move {
        if let Err(e) =
            shoot_cmd_infinitely(("127.0.0.1", SERVER_CMD_PORT), Command::Command, 10000).await
        {
            error!("Error in shoot command thread: {:?}", e);
        }
    });

    sleep().await;

    // TODO: not working
    // tokio::spawn(async move {
    //     if let Err(e) =
    //         listen_and_send_state((LOCAL_IP, SERVER_STATE_PORT), (TELLO_IP, TELLO_STATE_PORT)).await
    //     {
    //         error!("Error in listen state thread: {:?}", e);
    //     }
    // });

    tokio::spawn(async move {
        if let Err(e) = listen_and_stream_video(
            (LOCAL_IP, TELLO_VIDEO_STREAM_PORT),
            (TELLO_IP, TELLO_STREAM_ACCESS_PORT),
            &[
                ("127.0.0.1", 11112), // watchdog
                ("127.0.0.1", 11113), // detector
            ],
        )
        .await
        {
            error!("Error in listen command thread: {:?}", e);
        }
    });

    // メインスレッドが終了しないように待機
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
        info!("Waiting connection...");
        let (mut stream, addr) = match listener.accept().await {
            Ok(r) => r,
            Err(e) => return Err(Box::new(e)),
        };

        let dst_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();

        info!("Connected from {}", addr);

        tokio::spawn(async move {
            // receive cmd and send to dest target (is drone)
            let mut buf = vec![0; 1024];
            loop {
                let size = match stream.read(&mut buf).await {
                    Ok(size) => size,
                    Err(e) => {
                        error!("Error while reading from client ({}): {:?}", addr, e);
                        break;
                    }
                };

                if size == 0 {
                    break;
                }

                let data = &buf[..size];
                let s = String::from_utf8_lossy(data);

                let cmd = match Command::from_str(&s) {
                    Some(cmd) => cmd,
                    None => {
                        error!("Invalid command: \"{}\"", s);
                        stream.write_all("error".as_bytes()).await.unwrap();
                        continue;
                    }
                };

                info!("Receive command from client ({}): {:?}", addr, cmd);

                if let Err(e) = dst_socket
                    .send_to(cmd.to_string().as_bytes(), dst_target)
                    .await
                {
                    error!("Failed to send cmd to target: {:?}", e);
                    stream.write_all("error".as_bytes()).await.unwrap();
                    continue;
                }

                // wait response
                // TODO: timed out by from watchdog
                if tokio::time::timeout(Duration::from_millis(TIMEOUT_MS), async {
                    let size = match dst_socket.recv_from(&mut buf).await {
                        Ok((size, _)) => size,
                        Err(e) => {
                            error!("Failed to receive response from target: {:?}", e);
                            stream.write_all("error".as_bytes()).await.unwrap();
                            return;
                        }
                    };

                    let s = String::from_utf8_lossy(&buf[..size]);
                    info!("Receive response from target: {:?}", s);
                    stream.write_all(s.as_bytes()).await.unwrap();
                })
                .await
                .is_err()
                {
                    error!("Response timeouted from target");
                }
            }
            info!("End of connection with client ({})", addr);
        });
    }
}

async fn listen_stdin<A: tokio::net::ToSocketAddrs>(
    target: A,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = async_std::io::stdin();
    let mut line = String::new();

    let mut stream = TcpStream::connect(target).await?;

    loop {
        if stdin.read_line(&mut line).await.is_err() {
            continue;
        }

        stream.write_all(line.trim().as_bytes()).await?;
        line.clear();
    }
}

async fn shoot_cmd<A: tokio::net::ToSocketAddrs>(
    target: A,
    cmd: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(target).await?;
    stream.write_all(cmd.to_string().as_bytes()).await?;

    Ok(())
}

async fn shoot_cmd_infinitely<A: tokio::net::ToSocketAddrs>(
    target: A,
    cmd: Command,
    dur_ms: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(target).await?;
    let cmd_str = cmd.to_string();

    loop {
        stream.write_all(cmd_str.as_bytes()).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(dur_ms as u64)).await;
    }
}

async fn listen_and_send_state<A: tokio::net::ToSocketAddrs + Copy + Send + 'static>(
    listen_target: A,
    src_target: A,
) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(listen_target).await?;

    // UDPソケットをArcで包む
    let src_socket = Arc::new(Mutex::new(UdpSocket::bind(src_target).await.unwrap()));
    println!("{:?}", src_socket);

    // multi clients
    loop {
        info!("Waiting connection...");
        let (mut stream, addr) = match listener.accept().await {
            Ok(r) => r,
            Err(e) => return Err(Box::new(e)),
        };

        info!("Connected from {}", addr);

        // UDPソケットのクローンを作成
        let src_socket_clone = src_socket.clone();

        tokio::spawn(async move {
            // receive cmd and send to dest target (is drone)
            let mut buf = vec![0; 1024];

            loop {
                // UDPソケットへのアクセスをMutexで保護
                let src_socket_lock = src_socket_clone.lock().await;

                // wait response
                if tokio::time::timeout(Duration::from_millis(TIMEOUT_MS), async {
                    let size = match src_socket_lock.recv_from(&mut buf).await {
                        Ok((size, _)) => size,
                        Err(e) => {
                            error!("Failed to receive response from target: {:?}", e);
                            return;
                        }
                    };

                    let s = String::from_utf8_lossy(&buf[..size]).to_string();
                    let state = State::from_str(s.as_str());
                    info!("Receive state from target: {:?}", state);
                    stream
                        .write_all(serde_json::to_string(&state).unwrap().as_bytes())
                        .await
                        .unwrap();
                })
                .await
                .is_err()
                {
                    error!("Response timed out from target");
                }
            }
            //info!("End of connection with client ({})", addr);
        });
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
