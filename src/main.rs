use env_logger;
use log::error;
use std::{
    env,
    io::{self, BufRead},
};
use tello_autopilot::{tello::{cmd::Command, Tello}, server::Server};

fn main() {
    // env_logger setup
    // debug!, info!, warn!, error!

    // show info!, warn!, error! log
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let local_ip = "0.0.0.0"
        .parse()
        .expect("Failed to parse local ip address");
    let tello_ip = "192.168.10.1"
        .parse()
        .expect("Failed to parse tello ip address");
    let client_ip = "127.0.0.1"
        .parse()
        .expect("Failed to parse client ip address");

    let tello = Tello::new(300, local_ip, tello_ip);

    // 接続チェック
    if let Err(err) = tello.send_cmd(Command::Command, true) {
        error!("Error occured in send_cmd: {:?}", err);
        error!("Do check the connection with tello");
        //return;
    }

    let mut tcp_server = Server::new(client_ip, 8891);
    tcp_server.connect();
    tello.listen_state(tcp_server);

    // 入力ループ
    let mut input = String::new();
    loop {
        reader
            .read_line(&mut input)
            .expect("Failed to read line from stdin");

        if let Some(cmd) = Command::from_str(input.trim()) {
            if let Err(err) = tello.send_cmd(cmd, true) {
                error!("Error occured in send_cmd: {:?}", err);
            }
        } else {
            error!("Invalid command: \"{}\"", input.trim());
        }

        input.clear();
    }
}
