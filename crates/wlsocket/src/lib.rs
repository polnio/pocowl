use pocowl_wlclient::WaylandClient;
use pocowl_wlmessage::WaylandMessage;
pub use pocowl_wlvalue::WaylandValue;

use anyhow::{Context as _, Result};
use pocowl_protocols_base::WaylandProtocol;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use tokio::net::{UnixListener, UnixStream};

pub trait WaylandState {
    type ClientState: WaylandClientState;
    fn get_client_state_mut(&mut self, id: usize) -> Option<&mut Self::ClientState>;
    fn add_client(&mut self, id: usize);
}

pub trait WaylandClientState {
    fn get_protocol_of_object(&self, id: u32) -> Option<Rc<dyn WaylandProtocol<Self>>>;
}

pub struct WaylandSocket<State: WaylandState> {
    path: PathBuf,
    listener: UnixListener,
    state: State,

    last_client_id: usize,
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

            last_client_id: 0,
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

    async fn handle_connection(&mut self, stream: UnixStream) -> Result<()> {
        let mut client = WaylandClient {
            stream,
            id: self.last_client_id,
        };
        self.last_client_id += 1;
        self.state.add_client(client.id);
        let client_state = self
            .state
            .get_client_state_mut(client.id)
            .context("No client state found")?;
        let mut buf = VecDeque::<u8>::new();
        let mut temp_buf = [0u8; 512];
        loop {
            let n = Self::read_stream(&mut client.stream, &mut temp_buf).await?;
            if n == 0 {
                break;
            }
            buf.extend(&temp_buf[..n]);
            // let mut buf = &buf[..n];
            println!("C -> S: {:?}", buf);
            while !buf.is_empty() {
                let Ok(msg) = WaylandMessage::from_raw(&mut buf) else {
                    println!("{:?}", buf);
                    break;
                };
                println!("Got message: {msg:?}");
                let p = client_state
                    .get_protocol_of_object(msg.object_id)
                    .with_context(|| format!("No protocol found for object {}", msg.object_id))?;
                p.call(client_state, msg, &mut client);
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
