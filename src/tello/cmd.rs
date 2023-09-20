use std::fmt::{Display, Formatter, Result};

use super::state::State;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FlipCommandArg {
    Left,
    Right,
    Forward,
    Back,
}

impl FlipCommandArg {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "l" => Some(Self::Left),
            "r" => Some(Self::Right),
            "f" => Some(Self::Forward),
            "b" => Some(Self::Back),
            _ => None,
        }
    }
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

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Command,
    Takeoff,
    Land,
    StreamOn,
    StreamOff,
    Emergency,
    Up(usize),
    Down(usize),
    Left(usize),
    Right(usize),
    Forward(usize),
    Back(usize),
    ClockwiseRotation(usize),
    CounterClockwiseRotation(usize),
    Flip(FlipCommandArg),
    Go {
        x: isize,
        y: isize,
        z: isize,
        speed: usize,
        mid: Option<usize>,
    },
    Stop,
    Curve {
        x1: isize,
        y1: isize,
        z1: isize,
        x2: isize,
        y2: isize,
        z2: isize,
        speed: usize,
        mid: Option<usize>,
    },
    Jump {
        x: isize,
        y: isize,
        z: isize,
        speed: usize,
        yaw: usize,
        mid1: usize,
        mid2: usize,
    },
    Speed(usize),
    Rc {
        a: usize,
        b: usize,
        c: usize,
        d: usize,
    },
    Wifi {
        ssid: String,
        pass: String,
    },
    MissionpadOn,
    MissionpadOff,
    MissionpadDirection(usize),
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

impl Command {
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.trim().split_whitespace().collect();

        match parts.get(0) {
            Some(&"command") => Some(Command::Command),
            Some(&"takeoff") => Some(Command::Takeoff),
            Some(&"land") => Some(Command::Land),
            Some(&"streamon") => Some(Command::StreamOn),
            Some(&"streamoff") => Some(Command::StreamOff),
            Some(&"emergency") => Some(Command::Emergency),
            Some(&"up") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 20 && value <= 500 {
                    Some(Command::Up(value))
                } else {
                    None
                }
            }
            Some(&"down") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 20 && value <= 500 {
                    Some(Command::Down(value))
                } else {
                    None
                }
            }
            Some(&"left") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 20 && value <= 500 {
                    Some(Command::Left(value))
                } else {
                    None
                }
            }
            Some(&"right") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 20 && value <= 500 {
                    Some(Command::Right(value))
                } else {
                    None
                }
            }
            Some(&"forward") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 20 && value <= 500 {
                    Some(Command::Forward(value))
                } else {
                    None
                }
            }
            Some(&"back") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 20 && value <= 500 {
                    Some(Command::Back(value))
                } else {
                    None
                }
            }
            Some(&"cw") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 1 && value <= 360 {
                    Some(Command::ClockwiseRotation(value))
                } else {
                    None
                }
            }
            Some(&"ccw") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 1 && value <= 360 {
                    Some(Command::CounterClockwiseRotation(value))
                } else {
                    None
                }
            }
            Some(&"flip") if parts.len() == 2 => {
                let arg = FlipCommandArg::from_str(parts[1])?;
                Some(Command::Flip(arg))
            }
            Some(&"go") if parts.len() == 6 => {
                let x = parts[1].parse().ok()?;
                let y = parts[2].parse().ok()?;
                let z = parts[3].parse().ok()?;
                let speed = parts[4].parse().ok()?;
                Some(Command::Go {
                    x,
                    y,
                    z,
                    speed,
                    mid: None, // Note: Mid value not parsed
                })
            }
            Some(&"stop") => Some(Command::Stop),
            Some(&"curve") if parts.len() == 8 => {
                let x1 = parts[1].parse().ok()?;
                let y1 = parts[2].parse().ok()?;
                let z1 = parts[3].parse().ok()?;
                let x2 = parts[4].parse().ok()?;
                let y2 = parts[5].parse().ok()?;
                let z2 = parts[6].parse().ok()?;
                let speed = parts[7].parse().ok()?;
                Some(Command::Curve {
                    x1,
                    y1,
                    z1,
                    x2,
                    y2,
                    z2,
                    speed,
                    mid: None, // Note: Mid value not parsed
                })
            }
            Some(&"jump") if parts.len() == 9 => {
                // Note: Jump command not fully implemented
                Some(Command::Jump {
                    x: 0,
                    y: 0,
                    z: 0,
                    speed: 0,
                    yaw: 0,
                    mid1: 0,
                    mid2: 0,
                })
            }
            Some(&"speed") if parts.len() == 2 => {
                let value = parts[1].parse().ok()?;
                if value >= 10 && value <= 100 {
                    Some(Command::Speed(value))
                } else {
                    None
                }
            }
            Some(&"wifi") if parts.len() == 3 => {
                let ssid = parts[1].to_string();
                let pass = parts[2].to_string();
                Some(Command::Wifi { ssid, pass })
            }
            Some(&"mon") => Some(Command::MissionpadOn),
            Some(&"moff") => Some(Command::MissionpadOff),
            Some(&"ap") if parts.len() == 3 => {
                let ssid = parts[1].to_string();
                let pass = parts[2].to_string();
                Some(Command::AccessPoint { ssid, pass })
            }
            Some(&"speed?") => Some(Command::ReadSpeed),
            Some(&"battery?") => Some(Command::ReadBattery),
            Some(&"time?") => Some(Command::ReadTime),
            Some(&"wifi?") => Some(Command::ReadWifi),
            Some(&"sdk?") => Some(Command::ReadSdk),
            Some(&"sn?") => Some(Command::ReadSerialNumber),
            _ => None,
        }
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let s = match self {
            Self::Command => "command".to_string(),
            Self::Takeoff => "takeoff".to_string(),
            Self::Land => "land".to_string(),
            Self::StreamOn => "streamon".to_string(),
            Self::StreamOff => "streamoff".to_string(),
            Self::Emergency => "emergency".to_string(),
            Self::Up(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("up {}", value)
            }
            Self::Down(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("down {}", value)
            }
            Self::Left(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("left {}", value)
            }
            Self::Right(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("right {}", value)
            }
            Self::Forward(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("forward {}", value)
            }
            Self::Back(value) => {
                if !(*value >= 20 && *value <= 500) {
                    panic!("Not allowed argument: {:?}, must be 20 ~ 500", self);
                }

                format!("back {}", value)
            }
            Self::ClockwiseRotation(value) => {
                if !(*value >= 1 && *value <= 360) {
                    panic!("Not allowed argument: {:?}, must be 1 ~ 360", self);
                }

                format!("cw {}", value)
            }
            Self::CounterClockwiseRotation(value) => {
                if !(*value >= 1 && *value <= 360) {
                    panic!("Not allowed argument: {:?}, must be 1 ~ 360", self);
                }

                format!("ccw {}", value)
            }
            Self::Flip(value) => format!("flip {}", value),
            Self::Go {
                x,
                y,
                z,
                speed,
                mid,
            } => {
                if !(*x >= -500 && *x <= 500) {
                    panic!("Not allowed argument (x): {:?}, must be -500 ~ 500", self);
                }

                if !(*y >= -500 && *y <= 500) {
                    panic!("Not allowed argument (y): {:?}, must be -500 ~ 500", self);
                }

                if !(*z >= -500 && *z <= 500) {
                    panic!("Not allowed argument (z): {:?}, must be -500 ~ 500", self);
                }

                if !(*speed >= 10 && *speed <= 100) {
                    panic!("Not allowed argument (speed): {:?}, must be 10 ~ 100", self);
                }

                match mid {
                    Some(_) => unimplemented!(), // need missionpad
                    None => format!("go {} {} {} {}", x, y, z, speed),
                }
            }
            Self::Stop => "stop".to_string(),
            Self::Curve {
                x1,
                y1,
                z1,
                x2,
                y2,
                z2,
                speed,
                mid,
            } => {
                if !(*x1 >= -500 && *x1 <= 500) {
                    panic!("Not allowed argument (x1): {:?}, must be -500 ~ 500", self);
                }

                if !(*y1 >= -500 && *y1 <= 500) {
                    panic!("Not allowed argument (y1): {:?}, must be -500 ~ 500", self);
                }

                if !(*z1 >= -500 && *z1 <= 500) {
                    panic!("Not allowed argument (z1): {:?}, must be -500 ~ 500", self);
                }

                if !(*x2 >= -500 && *x2 <= 500) {
                    panic!("Not allowed argument (x2): {:?}, must be -500 ~ 500", self);
                }

                if !(*y2 >= -500 && *y2 <= 500) {
                    panic!("Not allowed argument (y2): {:?}, must be -500 ~ 500", self);
                }

                if !(*z2 >= -500 && *z2 <= 500) {
                    panic!("Not allowed argument (z2): {:?}, must be -500 ~ 500", self);
                }

                if !(*speed >= 10 && *speed <= 100) {
                    panic!("Not allowed argument (speed): {:?}, must be 10 ~ 100", self);
                }

                match mid {
                    Some(_) => unimplemented!(), // need missionpad
                    None => format!("curve {} {} {} {} {} {} {}", x1, y1, z1, x2, y2, z2, speed),
                }
            }
            Self::Jump {
                x: _,
                y: _,
                z: _,
                speed: _,
                yaw: _,
                mid1: _,
                mid2: _,
            } => unimplemented!(), // need missionpad
            Self::Speed(value) => {
                if !(*value >= 10 && *value <= 100) {
                    panic!("Not allowed argument: {:?}, must be 10 ~ 100", self);
                }

                format!("speed {}", value)
            }
            Self::Rc {
                a: _,
                b: _,
                c: _,
                d: _,
            } => unimplemented!(), // unknown command
            Self::Wifi { ssid, pass } => format!("wifi {} {}", ssid, pass),
            Self::MissionpadOn => "mon".to_string(),
            Self::MissionpadOff => "moff".to_string(),
            Self::MissionpadDirection(_) => todo!(),
            Self::AccessPoint { ssid, pass } => format!("ap {} {}", ssid, pass),
            Self::ReadSpeed => "speed?".to_string(),
            Self::ReadBattery => "battery?".to_string(),
            Self::ReadTime => "time?".to_string(),
            Self::ReadWifi => "wifi?".to_string(),
            Self::ReadSdk => "sdk?".to_string(),
            Self::ReadSerialNumber => "sn?".to_string(),
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
pub enum CommandResult {
    Ok,
    Error,
    State(State),
    Other(String),
}

impl CommandResult {
    pub fn from_str(s: &str) -> Self {
        match s {
            "ok" => CommandResult::Ok,
            "error" => CommandResult::Error,
            s => match State::from_str(s) {
                Some(state) => CommandResult::State(state),
                None => CommandResult::Other(s.to_string()),
            },
        }
    }
}
