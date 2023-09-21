use std::{
    io::{Error, ErrorKind},
    net::{IpAddr, UdpSocket},
    str,
    time::Duration,
};

pub mod cmd;
pub mod state;

use crate::tello::{
    cmd::{Command, CommandResult},
    state::State,
};

const TELLO_CMD_PORT: u16 = 8889;
const TELLO_STATE_PORT: u16 = 8890;
const TELLO_VIDEO_STREAM_PORT: u16 = 11111;
const SERVER_VIDEO_STREAM_PORT: u16 = 11112;
const VIDEO_STREAM_BUF_SIZE: usize = 1460;

pub struct Tello {
    timeout_dur: Duration,
    cmd_socket: UdpSocket,
    state_socket: UdpSocket,
    video_socket: UdpSocket,
    tello_ip: IpAddr,
}

impl Tello {
    pub fn new(
        timeout_millis: u64,
        local_ip: IpAddr,
        tello_ip: IpAddr,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            timeout_dur: Duration::from_millis(timeout_millis),
            cmd_socket: UdpSocket::bind((local_ip, TELLO_CMD_PORT))?,
            state_socket: UdpSocket::bind((local_ip, TELLO_STATE_PORT))?,
            video_socket: UdpSocket::bind((local_ip, TELLO_VIDEO_STREAM_PORT))?,
            tello_ip,
        })
    }

    pub fn connect(&self) -> Result<(), Error> {
        let dur = Some(self.timeout_dur);

        self.cmd_socket.set_broadcast(true)?;
        self.cmd_socket.set_read_timeout(dur)?;

        self.state_socket.set_broadcast(true)?;
        self.state_socket.set_read_timeout(dur)?;

        self.video_socket.set_broadcast(true)?;
        self.video_socket.set_read_timeout(dur)?;

        Ok(())
    }

    pub fn send_cmd(&self, cmd: Command, wait: bool) -> Result<Option<CommandResult>, Error> {
        let cmd_str = cmd.to_string();
        self.cmd_socket
            .send_to(cmd_str.as_bytes(), (self.tello_ip, TELLO_CMD_PORT))?;

        if !wait {
            return Ok(None);
        }

        // wait response
        let mut buf = [0; 1024];
        let (size, _) = self.cmd_socket.recv_from(&mut buf)?;
        let res_str = match str::from_utf8(&buf[..size]) {
            Ok(s) => s,
            Err(err) => return Err(Error::new(ErrorKind::Other, err)),
        };

        Ok(Some(CommandResult::from_str(res_str)))
    }

    pub fn receive_state(&self) -> Result<Option<State>, Error> {
        let mut buf = [0; 1024];
        let (size, _) = self.state_socket.recv_from(&mut buf)?;
        let res_str = match str::from_utf8(&buf[..size]) {
            Ok(s) => s,
            Err(err) => return Err(Error::new(ErrorKind::Other, err)),
        };

        Ok(State::from_str(res_str))
    }

    pub fn receive_and_send_video_stream(
        &self,
        buf: &mut [u8; VIDEO_STREAM_BUF_SIZE],
    ) -> Result<usize, Error> {
        let (size, _) = self.video_socket.recv_from(buf)?;
        self.video_socket
            .send_to(buf, ("127.0.0.1", SERVER_VIDEO_STREAM_PORT))?;
        Ok(size)
    }
}
