mod value;

pub use value::WaylandValue;

use anyhow::{Context as _, Result};
use byteorder::{NativeEndian, ReadBytesExt as _};
use pocowl_protocols_base::WaylandProtocol;
use std::path::PathBuf;
use std::rc::Rc;
use std::{io::Read as _, path::Path};
use tokio::net::{UnixListener, UnixStream};

pub trait WaylandState {
    fn get_protocol_of_object(&self, id: u32) -> Option<Rc<dyn WaylandProtocol<Self>>>;
}

#[derive(Debug)]
pub struct WaylandMessage {
    object_id: u32,
    opcode: u16,
    data: Vec<u8>,
}
impl WaylandMessage {
    fn from_raw(buf: &mut &[u8]) -> Result<Self> {
        const HEADER_LEN: usize = 8;
        if buf.len() < HEADER_LEN {
            return Err(anyhow::anyhow!("Invalid message: {} bytes", buf.len()));
        }
        let object_id = buf.read_u32::<NativeEndian>().unwrap();
        let opcode = buf.read_u16::<NativeEndian>().unwrap();
        let mut len = buf.read_u16::<NativeEndian>().unwrap();
        if len < 8 {
            return Err(anyhow::anyhow!(
                "length must be at least 8 bytes, got {}",
                len
            ));
        }
        len -= 8;

        let mut data = vec![0; len as usize];
        let m = buf.read(&mut data)?;
        if m != len as usize {
            return Err(anyhow::anyhow!(
                "length bigger than message size: {} > {} bytes",
                len,
                m
            ));
        }
        Ok(WaylandMessage {
            object_id,
            opcode,
            data,
        })
    }
}

pub struct WaylandSocket<State: WaylandState> {
    path: PathBuf,
    listener: UnixListener,
    state: State,
}
impl<State: WaylandState> WaylandSocket<State> {
    fn get_new_socket_path() -> Option<PathBuf> {
        let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
        let runtime_dir = PathBuf::from(runtime_dir);
        for display in 1..10 {
            let path = runtime_dir.join(format!("wayland-{}", display));
            if !path.exists() {
                return Some(path);
            }
        }
        None
    }

    pub fn create(state: State) -> Result<Self> {
        let path = Self::get_new_socket_path().context("No socket found")?;
        let listener = UnixListener::bind(&path).context("Failed to bind socket")?;
        Ok(Self {
            listener,
            path,
            state,
        })
    }

    pub async fn run(&mut self) {
        loop {
            let socket = self.listener.accept().await;
            let (stream, _) = socket.unwrap();
            if let Err(e) = self.handle_connection(stream).await {
                eprintln!("Failed to handle connection: {}", e);
            }
        }
    }

    async fn handle_connection(&mut self, mut stream: UnixStream) -> Result<()> {
        let mut buf = [0u8; 512];
        loop {
            let n = Self::read_stream(&mut stream, &mut buf).await?;
            if n == 0 {
                break;
            }
            let mut buf = &buf[..n];
            println!("{:?}", buf);
            while !buf.is_empty() {
                let msg = WaylandMessage::from_raw(&mut buf)?;
                println!("Got message: {msg:?}");
                let mut data = msg.data.as_slice();
                let p = self
                    .state
                    .get_protocol_of_object(msg.object_id)
                    .with_context(|| format!("No protocol found for object {}", msg.object_id))?;
                p.call(&mut self.state, msg.opcode, &mut data);
            }
        }
        println!("Connection closed");
        Ok(())
    }

    async fn read_stream(stream: &mut UnixStream, buf: &mut [u8]) -> Result<usize> {
        loop {
            stream.readable().await?;
            match stream.try_read(buf) {
                Ok(n) => return Ok(n),
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => {
                    return Err(e.into());
                }
            };
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl<State: WaylandState> Drop for WaylandSocket<State> {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}
