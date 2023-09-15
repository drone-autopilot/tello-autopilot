use env_logger;
use log::{error, info};
use std::{
    env,
    io::{self, BufRead},
};
use tello_autopilot::tello::{
    cmd::{Command, FlipCommandArg},
    Tello,
};

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

    let tello = Tello::new(300, local_ip, tello_ip);

    // 接続チェック
    if let Err(err) = tello.send_cmd(Command::Command, true) {
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

#[test]
fn cmd_test() {
    use tello_autopilot::tello::cmd::Command;

    assert_eq!(Command::Up(20).to_string(), "up 20");
    assert_eq!(
        Command::Curve {
            x1: -102,
            y1: 34,
            z1: 43,
            x2: -67,
            y2: 326,
            z2: 411,
            speed: 34,
            mid: None
        }
        .to_string(),
        "curve -102 34 43 -67 326 411 34"
    );

    assert!(Command::from_str("flip b").is_some());
}

#[test]
fn parse_state_test() {
    use tello_autopilot::tello::state::State;

    let s = "pitch:0;roll:2;yaw:0;vgx:0;vgy:0;vgz:0;templ:83;temph:85;tof:6553;h:0;bat:83;baro:193.06;time:0;agx:-5.00;agy:-48.00;agz:-998.00;\r\n";
    let state = State::from_str(s);

    assert!(state.is_some());
}
