pub use pocowl_wlvalue::WaylandValue;

use anyhow::{Context as _, Result};
use pocowl_protocols_base::WaylandProtocol;
use pocowl_wlclient::WaylandClient;
use pocowl_wlmessage::WaylandMessage;
use pocowl_wlstream::WaylandStream;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use tokio::net::{UnixListener, UnixStream};

pub trait WaylandState {
    type ClientState: WaylandClientState;
    fn get_client_state_mut(&mut self, id: usize) -> Option<&mut Self::ClientState>;
    fn add_client(&mut self, client: WaylandClient) -> &mut Self::ClientState;
}

pub trait WaylandClientState {
    fn get_client_mut(&mut self) -> &mut WaylandClient;
    fn get_protocol_of_object(&self, id: u32) -> Option<Rc<dyn WaylandProtocol<Self>>>;
    fn on_invalid_object(&mut self, id: u32);
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
            let result = self
                .handle_connection(stream)
                .await
                .context("Failed to handle connection");
            if let Err(e) = result {
                eprintln!("{e:?}");
            }
        }
    }

    async fn handle_connection(&mut self, stream: UnixStream) -> Result<()> {
        let id = self.last_client_id;
        let stream = stream.into_std().unwrap();
        let stream = WaylandStream::new(stream).context("Failed to create wayland stream")?;
        let client = WaylandClient::new(id, stream);
        self.last_client_id += 1;
        let client = self.state.add_client(client);
        let mut fds = VecDeque::new();
        loop {
            let Some(msg) = WaylandMessage::read(&mut client.get_client_mut().stream).await? else {
                break;
            };
            println!("Got message: {msg:?}");
            let sub_fds = client.get_client_mut().stream.fds();
            fds.extend(sub_fds.drain(..));
            let p = match client.get_protocol_of_object(msg.object_id) {
                Some(p) => p,
                None => {
                    client.on_invalid_object(msg.object_id);
                    continue;
                }
            };
            p.call(client, msg, &mut fds).await;
        }
        println!("Connection closed");
        Ok(())
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
