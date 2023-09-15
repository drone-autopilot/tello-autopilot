#[derive(Debug, Clone, Copy)]
pub struct PointState {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Default for PointState {
    fn default() -> Self {
        Self {
            x: 0f32,
            y: 0f32,
            z: 0f32,
        }
    }
}

// TODO: implement states for missionpad
#[derive(Debug, Clone)]
pub struct State {
    pub pitch: isize,
    pub roll: isize,
    pub yaw: isize,
    pub speeds: PointState,
    pub temp_low: usize,
    pub temp_high: usize,
    pub time_of_flight: usize,
    pub height: usize,
    pub battery: usize,
    pub barometer: f32,
    pub time: usize,
    pub accelerations: PointState,
}

impl Default for State {
    fn default() -> Self {
        Self {
            pitch: 0,
            roll: 0,
            yaw: 0,
            speeds: PointState::default(),
            temp_low: 0,
            temp_high: 0,
            time_of_flight: 0,
            height: 0,
            battery: 0,
            barometer: 0f32,
            time: 0,
            accelerations: PointState::default(),
        }
    }
}

impl State {
    pub fn from_str(s: &str) -> Option<Self> {
        let mut state = Self::default();

        let fields: Vec<&str> = s.trim().split(';').collect();

        if fields.len() != 17 {
            return None;
        }

        let mut point_state = PointState::default();

        for field in fields {
            let parts: Vec<&str> = field.split(':').collect();

            if parts.len() != 2 {
                continue;
            }

            match parts[0] {
                "pitch" => state.pitch = parts[1].parse().ok()?,
                "roll" => state.roll = parts[1].parse().ok()?,
                "yaw" => state.yaw = parts[1].parse().ok()?,
                "vgx" => point_state.x = parts[1].parse().ok()?,
                "vgy" => point_state.y = parts[1].parse().ok()?,
                "vgz" => {
                    point_state.z = parts[1].parse().ok()?;
                    state.speeds = point_state;
                    point_state = PointState::default();
                }
                "templ" => state.temp_low = parts[1].parse().ok()?,
                "temph" => state.temp_high = parts[1].parse().ok()?,
                "tof" => state.time_of_flight = parts[1].parse().ok()?,
                "h" => state.height = parts[1].parse().ok()?,
                "bat" => state.battery = parts[1].parse().ok()?,
                "baro" => state.barometer = parts[1].parse().ok()?,
                "time" => state.time = parts[1].parse().ok()?,
                "agx" => point_state.x = parts[1].parse().ok()?,
                "agy" => point_state.y = parts[1].parse().ok()?,
                "agz" => {
                    point_state.z = parts[1].parse().ok()?;
                    state.accelerations = point_state;
                    point_state = PointState::default();
                }
                _ => (),
            }
        }

        return Some(state);
    }
}
