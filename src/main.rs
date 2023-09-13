use std::io::{self, BufRead};
use tello_autopilot::tello::Tello;

fn main() {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let local_ip = "0.0.0.0"
        .parse()
        .expect("Failed to parse local ip address");
    let tello_ip = "192.168.10.1"
        .parse()
        .expect("Failed to parse tello ip address");

    // 接続チェック
    let tello = Tello::new(3, local_ip, tello_ip);
    let result = tello.send_cmd("command", true);

    if !result {
        println!("Telloとの接続を確認してください.");
    }

    tello.listen_state();

    // 入力ループ
    loop {
        let mut input = String::new();

        match reader.read_line(&mut input) {
            Ok(0) => {
                break;
            }
            Ok(_) => {
                let result = tello.send_cmd(&input.lines().collect::<String>(), true);
                if !result {
                    eprintln!("Failed to send command: {}", &input);
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }
}