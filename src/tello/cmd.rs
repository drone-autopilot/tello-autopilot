use std::fmt::{Display, Formatter, Result};

#[derive(Debug, Clone, Copy)]
pub enum FlipCommandArg {
    Left,
    Right,
    Forward,
    Back,
}

impl Display for FlipCommandArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = match self {
            Self::Left => "l",
            Self::Right => "r",
            Self::Forward => "f",
            Self::Back => "b",
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub enum Command {
    Command,
    Takeoff,
    Land,
    StreamOn,
    StreamOff,
    Emergency,
    Up(u16),
    Down(u16),
    Left(u16),
    Right(u16),
    Forward(u16),
    Back(u16),
    ClockwiseRotation(u16),
    CounterClockwiseRotation(u16),
    Flip(FlipCommandArg),
    Go {
        x: i16,
        y: i16,
        z: i16,
        speed: u8,
        mid: Option<u8>,
    },
    Stop,
    Curve {
        x1: i16,
        y1: i16,
        z1: i16,
        x2: i16,
        y2: i16,
        z2: i16,
        speed: u8,
        mid: Option<u8>,
    },
    Jump {
        x: i16,
        y: i16,
        z: i16,
        speed: u8,
        yaw: u8,
        mid1: u8,
        mid2: u8,
    },
    Speed(u8),
    Rc {
        a: u8,
        b: u8,
        c: u8,
        d: u8,
    },
    Wifi {
        ssid: String,
        pass: String,
    },
    MissionpadOn,
    MissionpadOff,
    MissionpadDirection(u8),
    AccessPoint {
        ssid: String,
        pass: String,
    },
    ReadSpeed,
    ReadBattery,
    ReadTime,
    ReadWifi,
    ReadSdk,
    ReadSerialNumber,
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = match self {
            Command::Command => "command".to_string(),
            Command::Takeoff => "takeoff".to_string(),
            Command::Land => "land".to_string(),
            Command::StreamOn => "streamon".to_string(),
            Command::StreamOff => "streamoff".to_string(),
            Command::Emergency => "emergency".to_string(),
            Command::Up(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("up {}", value)
            }
            Command::Down(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("down {}", value)
            }
            Command::Left(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("left {}", value)
            }
            Command::Right(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("right {}", value)
            }
            Command::Forward(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("forward {}", value)
            }
            Command::Back(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("back {}", value)
            }
            Command::ClockwiseRotation(value) => {
                if !(*value >= 1 && *value <= 360) {
                    panic!("Not allowed argument: {:?}, must be 1 ~ 360", self);
                }

                format!("cw {}", value)
            }
            Command::CounterClockwiseRotation(value) => {
                if !(*value >= 1 && *value <= 360) {
                    panic!("Not allowed argument: {:?}, must be 1 ~ 360", self);
                }

                format!("ccw {}", value)
            }
            Command::Flip(value) => format!("flip {}", value),
            Command::Go {
                x,
                y,
                z,
                speed,
                mid,
            } => todo!(),
            Command::Stop => "stop".to_string(),
            Command::Curve {
                x1,
                y1,
                z1,
                x2,
                y2,
                z2,
                speed,
                mid,
            } => todo!(),
            Command::Jump {
                x,
                y,
                z,
                speed,
                yaw,
                mid1,
                mid2,
            } => todo!(),
            Command::Speed(value) => {
                if !(*value >= 10 && *value <= 100) {
                    panic!("Not allowed argument: {:?}, must be 10 ~ 100", self);
                }

                format!("speed {}", value)
            }
            Command::Rc { a, b, c, d } => todo!(),
            Command::Wifi { ssid, pass } => todo!(),
            Command::MissionpadOn => "mon".to_string(),
            Command::MissionpadOff => "moff".to_string(),
            Command::MissionpadDirection(_) => todo!(),
            Command::AccessPoint { ssid, pass } => todo!(),
            Command::ReadSpeed => "speed?".to_string(),
            Command::ReadBattery => "battery?".to_string(),
            Command::ReadTime => "time?".to_string(),
            Command::ReadWifi => "wifi?".to_string(),
            Command::ReadSdk => "sdk?".to_string(),
            Command::ReadSerialNumber => "sn?".to_string(),
        };

        write!(f, "{}", s)
    }
}
