use log::{error, info};
use std::env;
use tello_autopilot::{cmd::Command, state::State};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, UdpSocket},
};

const LOCAL_IP: &'static str = "0.0.0.0";
//const TELLO_IP: &'static str = "192.168.10.1";
const TELLO_IP: &'static str = "127.0.0.1"; // for debugging
const WATCHDOG_IP: &'static str = "127.0.0.1";

const TELLO_CMD_PORT: u16 = 8889;
const TELLO_STATE_PORT: u16 = 8890;
const TELLO_VIDEO_STREAM_PORT: u16 = 11111;
const TELLO_STREAM_ACCESS_PORT: u16 = 62512;

const SERVER_CMD_PORT: u16 = 8989;
const SERVER_STATE_PORT: u16 = 8990;
const SERVER_VIDEO_STREAM_PORT: u16 = 11112;

const PILOT_VIDEO_STREAM_PORT: u16 = 11113;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    tokio::spawn(async move {
        if let Err(e) =
            listen_and_send_cmd((LOCAL_IP, SERVER_CMD_PORT), (TELLO_IP, TELLO_CMD_PORT)).await
        {
            error!("Error in listen command thread: {:?}", e);
        }
    });

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

    // tokio::spawn(async move {
    //     if let Err(e) = listen_and_stream_video(
    //         (LOCAL_IP, TELLO_VIDEO_STREAM_PORT),
    //         (TELLO_IP, TELLO_STREAM_ACCESS_PORT),
    //         &[
    //             (WATCHDOG_IP, SERVER_VIDEO_STREAM_PORT),
    //             (WATCHDOG_IP, PILOT_VIDEO_STREAM_PORT),
    //         ],
    //     )
    //     .await
    //     {
    //         error!("Error in listen command thread: {:?}", e);
    //     }
    // });

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

        info!("Connected from {}", addr);

        tokio::spawn(async move {
            // receive cmd and send to dest target (is drone)
            let mut buf = vec![0; 1024];
            loop {
                //info!("Waiting message from client");
                match stream.read(&mut buf).await {
                    Ok(size) => {
                        if size == 0 {
                            break;
                        }

                        let data = &buf[..size];
                        let s = String::from_utf8_lossy(data);

                        match Command::from_str(&s) {
                            Some(cmd) => {
                                info!("Receive command from client ({}): {:?}", addr, cmd);

                                let dst_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
                                if let Err(e) = dst_socket
                                    .send_to(cmd.to_string().as_bytes(), dst_target)
                                    .await
                                {
                                    error!("Failed to send cmd to target: {:?}", e);
                                    stream.write_all("error".as_bytes()).await.unwrap();
                                    continue;
                                }

                                // wait response
                                // TODO: timeout
                                let size = match dst_socket.recv_from(&mut buf).await {
                                    Ok((size, _)) => size,
                                    Err(e) => {
                                        error!("Failed to receive response from target: {:?}", e);
                                        stream.write_all("error".as_bytes()).await.unwrap();
                                        continue;
                                    }
                                };

                                let s = String::from_utf8_lossy(&buf[..size]);
                                info!("Receive response from target: {:?}", s);
                                stream.write_all(s.as_bytes()).await.unwrap();
                            }
                            None => {
                                error!("Invalid command: \"{}\"", s);
                                stream.write_all("error".as_bytes()).await.unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error while reading from client ({}): {:?}", addr, e);
                        break;
                    }
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

// TODO
//async fn listen_and_send_state<A: tokio::net::ToSocketAddrs + Copy>(
//     listen_target: A,
//     src_target: A,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let listener = TcpListener::bind(listen_target).await?;

//     info!("Connecting to src target...");
//     let src_socket = UdpSocket::bind(src_target).await?;
//     let mut buf = vec![0; 1024];

//     // multi clients
//     loop {
//         info!("Waiting connection...");
//         let (mut stream, addr) = match listener.accept().await {
//             Ok(r) => r,
//             Err(e) => return Err(Box::new(e)),
//         };

//         info!("Connected from {}", addr);
//         tokio::spawn(async move {
//             loop {
//                 match src_socket.recv_from(&mut buf).await {
//                     Ok((size, _)) => {
//                         // ignore errors
//                         stream.write_all("ok".as_bytes()).await.unwrap();
//                     }
//                     Err(e) => {
//                         error!("Error while listening video: {:?}", e);
//                     }
//                 }
//             }
//         });
//     }
// }

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
            Err(e) => {
                error!("Error while listening video: {:?}", e);
            }
        }
    }
}
