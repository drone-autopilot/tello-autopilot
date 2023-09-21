use crate::tello::state::State;
use std::{
    io::{Error, ErrorKind, Read, Write},
    net::{IpAddr, TcpListener, TcpStream, UdpSocket},
    str,
    time::Duration,
};

const SERVER_CMD_PORT: u16 = 8989;
const SERVER_STATE_PORT: u16 = 8990;
const SERVER_VIDEO_STREAM_PORT: u16 = 11112;
const VIDEO_STREAM_BUF_SIZE: usize = 1460;

pub struct WatchdogServer {
    timeout_dur: Duration,
    state_stream: Option<TcpStream>,
    cmd_stream: Option<TcpStream>,
    video_socket: UdpSocket,
    client_ip: IpAddr,
}

impl WatchdogServer {
    pub fn new(timeout_millis: u64, client_ip: IpAddr) -> Result<Self, std::io::Error> {
        Ok(Self {
            timeout_dur: Duration::from_millis(timeout_millis),
            state_stream: None,
            cmd_stream: None,
            video_socket: UdpSocket::bind("0.0.0.0:0")?,
            client_ip,
        })
    }

    pub fn wait_for_connection(&mut self) -> Result<(), Error> {
        let dur = Some(self.timeout_dur);

        let state_socket = TcpListener::bind((self.client_ip, SERVER_STATE_PORT))?;
        let (stream, _) = state_socket.accept()?;
        stream.set_write_timeout(dur)?;
        self.state_stream = Some(stream);

        let cmd_socket = TcpListener::bind((self.client_ip, SERVER_CMD_PORT))?;
        let (stream, _) = cmd_socket.accept()?;
        stream.set_read_timeout(dur)?;
        self.cmd_stream = Some(stream);

        self.video_socket.set_broadcast(true)?;
        self.video_socket.set_read_timeout(Some(self.timeout_dur))?;

        Ok(())
    }

    pub fn send_video_stream(&self, buf: &[u8; VIDEO_STREAM_BUF_SIZE]) -> Result<(), Error> {
        self.video_socket
            .send_to(buf, (self.client_ip, SERVER_VIDEO_STREAM_PORT))?;

        Ok(())
    }

    pub fn send_tello_state(&self, state: State) -> Result<(), Error> {
        self.state_stream()
            .write_all(serde_json::to_string(&state).unwrap().as_bytes())?;
        Ok(())
    }

    pub fn receive_cmd(&self) -> Result<String, Error> {
        let mut buf = [0; 1024];
        let size = self.cmd_stream().read(&mut buf)?;

        let rec_str = match str::from_utf8(&buf[..size]) {
            Ok(s) => s,
            Err(err) => return Err(Error::new(ErrorKind::Other, err)),
        }
        .to_string();

        Ok(rec_str)
    }

    fn state_stream(&self) -> &TcpStream {
        self.state_stream.as_ref().unwrap()
    }

    fn cmd_stream(&self) -> &TcpStream {
        self.cmd_stream.as_ref().unwrap()
    }
}
