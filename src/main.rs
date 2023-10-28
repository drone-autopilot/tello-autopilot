use log::{error, info};
use std::env;
use tello_autopilot::{cmd::Command, state::State};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream, ToSocketAddrs, UdpSocket},
    signal::ctrl_c,
    spawn,
    time::{sleep, timeout, Duration},
};

type Addr = (&'static str, u16);

const LISTEN_CMD_ADDR: Addr = ("127.0.0.1", 8989);
const LISTEN_STATE_ADDR: Addr = ("127.0.0.1", 8990);

const TELLO_CMD_ADDR: Addr = ("192.168.10.1", 8889);
const TELLO_STATE_ADDR: Addr = ("0.0.0.0", 8890);
const TELLO_VIDEO_ADDR: Addr = ("0.0.0.0", 11111);
const TELLO_VIDEO_DOORBELL_ADDR: Addr = ("192.168.10.1", 62512);

const TIMEOUT_MS: u64 = 15000; // 15s
const RES_TIMEOUT_MS: u64 = 3000; // 3s

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async fn sleep_5s() {
        sleep(Duration::from_millis(5000)).await;
    }

    // TODO: not working
    // TODO: shoot_cmd -> shoot_cmd_infinitely -> listen_state
    // NOTE: おそらくdst_sokcetをクライアントごとに再生成しているせい -> これを共通で扱いたい
    // fn send_cmd(cmd: Command) {
    //     spawn(async move {
    //         if let Err(e) = shoot_cmd(LISTEN_CMD_ADDR, &cmd).await {
    //             error!("sned cmd: {:?}", e);
    //         }
    //     });
    // }

    env::set_var("RUST_LOG", "info");
    env_logger::init();

    // command
    spawn(async move {
        if let Err(e) = listen_and_send_cmd(LISTEN_CMD_ADDR, TELLO_CMD_ADDR).await {
            error!("listen cmd: {:?}", e);
        }
    });

    sleep_5s().await;

    //send_cmd(Command::Command);
    //send_cmd(Command::StreamOn);

    spawn(async move {
        if let Err(e) = listen_stdin(LISTEN_CMD_ADDR).await {
            error!("listen stdin: {:?}", e);
        }
    });

    // spawn(async move {
    //     if let Err(e) = shoot_cmd_infinitely(LISTEN_CMD_ADDR, &Command::Command, 4000).await {
    //         error!("listen shoot cmd: {:?}", e);
    //     }
    // });

    // spawn(async move {
    //     if let Err(e) = listen_and_send_state(LISTEN_STATE_ADDR, TELLO_STATE_ADDR).await {
    //         error!("Error in listen state thread: {:?}", e);
    //     }
    // });

    // video
    //send_cmd(Command::StreamOn);

    spawn(async move {
        if let Err(e) = listen_and_stream_video(
            TELLO_VIDEO_ADDR,
            TELLO_VIDEO_DOORBELL_ADDR,
            &[
                ("127.0.0.1", 11112), // watchdog
                ("127.0.0.1", 11113), // detector
            ],
        )
        .await
        {
            error!("listen video: {:?}", e);
        }
    });

    ctrl_c().await?;

    Ok(())
}

async fn listen_and_send_cmd<A: ToSocketAddrs + Copy + Send + 'static>(
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

        let mut buf = vec![0; 1024];
        let dst_socket = UdpSocket::bind("0.0.0.0:0").await.unwrap();
        info!("listen cmd: Connected from {}", addr);

        spawn(async move {
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
                    continue;
                }

                // wait response
                if timeout(Duration::from_millis(RES_TIMEOUT_MS), async {
                    let size = match dst_socket.recv_from(&mut buf).await {
                        Ok((size, _)) => size,
                        Err(e) => {
                            error!(
                                "listen cmd: Failed to receive response from target: {:?}",
                                e
                            );
                            stream.write_all("error".as_bytes()).await.unwrap();
                            return;
                        }
                    };

                    let s = String::from_utf8_lossy(&buf[..size]);
                    info!("listen cmd: Receive response from target: {:?}", s);
                    stream.write_all(s.as_bytes()).await.unwrap();
                })
                .await
                .is_err()
                {
                    error!("Timed out");
                    stream.write_all("error".as_bytes()).await.unwrap();
                }
            }
            info!("listen cmd: End of connection with client ({})", addr);
        });
    }
}

async fn listen_stdin<A: ToSocketAddrs + Copy>(
    target: A,
) -> Result<(), Box<dyn std::error::Error>> {
    let stdin = async_std::io::stdin();
    let mut line = String::new();
    let mut buf = vec![0; 128];

    let mut stream = TcpStream::connect(target).await?;

    loop {
        if stdin.read_line(&mut line).await.is_err() {
            continue;
        }

        stream.write_all(line.trim().as_bytes()).await?;
        stream.read_buf(&mut buf).await?;
        line.clear();
    }
}

// async fn shoot_cmd<A: ToSocketAddrs>(
//     target: A,
//     cmd: &Command,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     let mut buf = vec![0; 128];
//     let mut stream = TcpStream::connect(target).await?;
//     stream.write_all(cmd.to_string().as_bytes()).await?;
//     stream.read_buf(&mut buf).await?;

//     Ok(())
// }

// async fn shoot_cmd_infinitely<A: ToSocketAddrs + Copy>(
//     target: A,
//     cmd: &Command,
//     dur_ms: u64,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     if dur_ms < RES_TIMEOUT_MS {
//         return Err(format!("dur_ms is shorter than {}ms", RES_TIMEOUT_MS).into());
//     }

//     let mut stream = TcpStream::connect(target).await?;
//     let mut buf = vec![0; 128];
//     let s = cmd.to_string();

//     loop {
//         stream.write_all(s.as_bytes()).await?;
//         stream.read_buf(&mut buf).await?;
//         sleep(Duration::from_millis(dur_ms)).await;
//     }
// }

// async fn listen_and_send_state<A: ToSocketAddrs + Copy + Send + 'static>(
//     tcp_listen_target: A,
//     udp_listen_target: A,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     //let listener = TcpListener::bind(listen_target).await?;
//     let src_server_socket = UdpSocket::bind(udp_listen_target).await?;

//     let mut buf = vec![0; 1024];

//     loop {
//         info!("listen state: Waiting...");
//         let size = match src_server_socket.recv_from(&mut buf).await {
//             Ok((size, _)) => size,
//             Err(e) => {
//                 error!(
//                     "listen state: Failed to receive response from target: {:?}",
//                     e
//                 );
//                 continue;
//             }
//         };

//         let s = String::from_utf8_lossy(&buf[..size]).to_string();
//         let state = State::from_str(s.as_str());
//         info!("listen state: Receive state from target: {:?}", state);
//     }
// }

async fn listen_and_stream_video<A: ToSocketAddrs>(
    listen_target: A,
    doorbell_target: A,
    dst_target: &[A],
) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind(listen_target).await?;
    let mut buf = vec![0; 1460];

    socket.send_to(b"", doorbell_target).await?;

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
