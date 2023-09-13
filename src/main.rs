use std::{io, process::exit, thread};
use tello_autopilot::tello::Tello;

fn main() {
    let local_ip = "192.168.10.2";
    let tello_ip = "192.168.10.1";
    let tello = Tello::new(15, local_ip, tello_ip);

    // 接続チェック
    let result = tello.send_cmd("command", true);
    if !result {
        println!("Telloとの接続を確認してください。");
        exit(0);
    }
    tello.listen_state();

    // 入力ループ
    let handle = thread::spawn(move || loop {
        let mut str = String::new();
        io::stdin()
            .read_line(&mut str)
            .expect("failed to read line");
        tello.send_cmd(&str.lines().collect::<String>(), false);
    });

    handle.join().unwrap();
}
