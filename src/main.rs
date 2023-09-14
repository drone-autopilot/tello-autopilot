use std::io::{self, BufRead};
use tello_autopilot::tello::Tello;

fn main() {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let local_ip = "0.0.0.0".parse().expect("Failed to parse local ip address");
    let tello_ip = "192.168.10.1"
        .parse()
        .expect("Failed to parse tello ip address");

    // 接続チェック
    let tello = Tello::new(300, local_ip, tello_ip);

    if let Err(err) = tello.send_cmd("command", true) {
        eprintln!("Error occured in send_cmd: {:?}", err);
        eprintln!("Do check the connection with tello");
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
            eprintln!("Error occured in send_cmd: {:?}", err);
        }

        input.clear();
    }
}
