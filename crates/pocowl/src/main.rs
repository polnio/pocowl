mod protocols;
mod value;

use crate::protocols::WaylandProtocol;
use crate::protocols::wayland::{WlDisplay, WlDisplayListener, WlRegistry, WlRegistryListener};
use anyhow::{Context, Result};
use byteorder::{NativeEndian, ReadBytesExt};
use std::path::PathBuf;
use std::rc::Rc;
use std::{collections::HashMap, io::Read as _};
use tokio::net::{UnixListener, UnixStream};
use tokio_util::sync::CancellationToken;

struct PocoWl {
    objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>>,
}
impl PocoWl {
    fn new() -> Self {
        let mut objects: HashMap<u32, Rc<dyn WaylandProtocol<Self>>> = HashMap::new();
        objects.insert(1, Rc::new(WlDisplay));
        Self { objects }
    }
}

impl WlRegistryListener for PocoWl {
    fn bind(&mut self, name: u32, id: u32) -> u32 {
        todo!()
    }
}

impl WlDisplayListener for PocoWl {
    fn sync(&mut self, callback: u32) -> u32 {
        // todo!()
        0
    }

    fn get_registry(&mut self, registry: u32) -> u32 {
        self.objects.insert(registry, Rc::new(WlRegistry));
        0
    }
}

// fn get_transport() -> Option<PathBuf> {
//     let display = std::env::var_os("WAYLAND_DISPLAY");
//     let display = display.as_deref().unwrap_or("wayland-0".as_ref());
//     let runtime_dir = std::env::var_os("XDG_RUNTIME_DIR")?;
//     let path = PathBuf::from(runtime_dir).join(display);
//     Some(path)
// }

#[derive(Debug)]
struct WaylandMessage {
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

struct WaylandSocket {
    path: PathBuf,
    listener: UnixListener,
}
impl WaylandSocket {
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

    fn create() -> Result<Self> {
        let path = Self::get_new_socket_path().context("No socket found")?;
        let listener = UnixListener::bind(&path).context("Failed to bind socket")?;
        Ok(Self { listener, path })
    }

    async fn run(&mut self) {
        loop {
            let socket = self.listener.accept().await;
            let (stream, _) = socket.unwrap();
            if let Err(e) = Self::handle_connection(stream).await {
                eprintln!("Failed to handle connection: {}", e);
            }
        }
    }

    async fn handle_connection(mut stream: UnixStream) -> Result<()> {
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
                let mut state = PocoWl::new();
                let mut data = msg.data.as_slice();
                let p = state.objects.get(&msg.object_id).cloned().unwrap();
                p.call(&mut state, msg.opcode, &mut data);
                // protocols::wayland::WlDisplay::call(&state, msg.opcode, &mut buf);
                // state.call(msg.opcode, &mut buf);
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
}

impl Drop for WaylandSocket {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

async fn run_with_cancellation<F>(f: F)
where
    F: Future<Output = ()> + Send + 'static,
{
    let cancel = CancellationToken::new();
    let cancel_handle = tokio::spawn({
        let cancel = cancel.clone();
        async move {
            let _ = tokio::signal::ctrl_c().await;
            cancel.cancel();
        }
    });

    let server_handle = tokio::spawn(async move { cancel.run_until_cancelled(f).await });
    let _ = tokio::join!(cancel_handle, server_handle);
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut socket = WaylandSocket::create()?;
    println!("Listening on {}", socket.path.display());

    run_with_cancellation(async move { socket.run().await }).await;
    Ok(())
}
