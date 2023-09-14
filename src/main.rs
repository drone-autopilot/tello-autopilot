use env_logger;
use log::error;
use std::{
    env,
    io::{self, BufRead},
};
use tello_autopilot::tello::Tello;

fn main() {
    // env_logger setup
    // debug!, info!, warn!, error!

    // show info!, warn!, error! log
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let local_ip = "0.0.0.0".parse().expect("Failed to parse local ip address");
    let tello_ip = "192.168.10.1"
        .parse()
        .expect("Failed to parse tello ip address");

    // 接続チェック
    let tello = Tello::new(300, local_ip, tello_ip);

    if let Err(err) = tello.send_cmd("command", true) {
        error!("Error occured in send_cmd: {:?}", err);
        error!("Do check the connection with tello");
        //return;
    }

    tello.listen_state();

    // 入力ループ
    let mut input = String::new();
    loop {
        reader
            .read_line(&mut input)
            .expect("Failed to read line from stdin");

        if let Err(err) = tello.send_cmd(&input.lines().collect::<String>(), true) {
            error!("Error occured in send_cmd: {:?}", err);
        }

        input.clear();
    }
}
