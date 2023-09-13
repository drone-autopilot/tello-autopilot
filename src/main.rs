use tello_autopilot::tello::Tello;
use std::error::Error;
use std::io::{self, BufRead};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut reader = stdin.lock();

    let local_ip = "192.168.10.2";
    let tello_ip = "192.168.10.1";

    // 接続チェック
    let result = tokio::task::spawn_blocking(move || {
        let tello = Tello::new(3, local_ip, tello_ip);
        tello.send_cmd("command", true)
    }).await.expect("failed to spawn blocking task");

    if !result {
        println!("Telloとの接続を確認してください.");
    }

    // 入力ループ
    loop {
        let mut input = String::new();

        match reader.read_line(&mut input) {
            Ok(0) => {
                break;
            }
            Ok(_) => {
                // ここで tello を再度所有権移動
                let tello = Tello::new(3, local_ip, tello_ip);
                let _ = tello.send_cmd(&input.lines().collect::<String>(), false);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}