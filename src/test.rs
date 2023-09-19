#[test]
fn cmd_test() {
    use crate::tello::cmd::Command;
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
    use crate::tello::state::State;
    let s = "pitch:0;roll:2;yaw:0;vgx:0;vgy:0;vgz:0;templ:83;temph:85;tof:6553;h:0;bat:83;baro:193.06;time:0;agx:-5.00;agy:-48.00;agz:-998.00;\r\n";
    let state = State::from_str(s);

    assert!(state.is_some());
}
