use crate::protocols::xdg_shell::XdgWmBase;
use crate::protocols::{DISPLAY_OBJECT, WaylandProtocol, wayland::*};
use crate::socket::Client;
use crate::utils::WaylandBuffer;
use async_trait::async_trait;
use memmap::MmapMut;
use std::collections::HashMap;
use std::os::fd::OwnedFd;
use tokio::io::AsyncWriteExt as _;
use tracing::{error, warn};

const SUPPORTED_INTERFACE_FACTORIES: [fn(u32) -> Box<dyn WaylandProtocol<Client> + Send>; 4] = [
    |id| Box::new(WlCompositor { object_id: id }),
    |id| Box::new(WlShm { object_id: id }),
    |id| Box::new(WlOutput { object_id: id }),
    |id| Box::new(XdgWmBase { object_id: id }),
];

struct ImpWlSurface {
    inner: WlSurface,
    buffer: Option<WlBuffer>,
}
struct ImpWlBuffer {
    inner: WlBuffer,
    // buf: WaylandBuffer,
    width: usize,
    height: usize,
}
struct ImpWlShmPool {
    inner: WlShmPool,
    buffers: Vec<WlBuffer>,
    mmap: MmapMut,
}
#[derive(Default)]
pub struct ImpWaylandState {
    surfaces: HashMap<WlSurface, ImpWlSurface>,
    buffers: HashMap<WlBuffer, ImpWlBuffer>,
    shm_pools: HashMap<WlShmPool, ImpWlShmPool>,
}

impl Client {
    fn wl_state(&self) -> &ImpWaylandState {
        &self.imp_proto_states.wayland
    }
    fn wl_state_mut(&mut self) -> &mut ImpWaylandState {
        &mut self.imp_proto_states.wayland
    }
}

#[async_trait]
impl WlDisplayListener for Client {
    async fn sync(&mut self, object: WlDisplay, callback: WlCallback) {
        _ = object;
        let mut data = Vec::new();
        // data.extend(WlDisplay::delete_id(object, callback.object_id).to_raw());
        data.extend(callback.done(Default::default()).to_raw());
        let _ = self.stream.write(&data).await;
    }

    async fn get_registry(&mut self, object: WlDisplay, registry: WlRegistry) {
        _ = object;
        // self.objects.insert(registry.object_id, Box::new(registry));
        self.add_object(registry);
        let mut data = Vec::new();
        for (name, interface_factory) in SUPPORTED_INTERFACE_FACTORIES.iter().enumerate() {
            let interface = (interface_factory)(registry.object_id);
            data.extend(
                registry
                    .global(
                        name as u32,
                        interface.name().to_owned(),
                        interface.version(),
                    )
                    .to_raw(),
            );
        }
        let _ = self.stream.write(&data).await;
    }
}

#[async_trait]
impl WlRegistryListener for Client {
    async fn bind(
        &mut self,
        object: WlRegistry,
        name: u32,
        id_interface: String,
        id_version: u32,
        id: u32,
    ) {
        let Some(interface_factory) = SUPPORTED_INTERFACE_FACTORIES.get(name as usize) else {
            self.error(
                object.object_id,
                WlDisplayError::InvalidObject as u32,
                format!("Invalid interface name: {}", name),
            )
            .await;
            return;
        };
        let interface = (interface_factory)(id);
        if id_interface != interface.name() {
            self.error(
                object.object_id,
                WlDisplayError::InvalidObject as u32,
                format!(
                    "Expected interface {}, found {}",
                    interface.name(),
                    id_interface
                ),
            )
            .await;
            return;
        }
        if id_version > interface.version() {
            self.error(
                object.object_id,
                WlDisplayError::InvalidObject as u32,
                format!(
                    "Expected interface version <= {}, found {}",
                    interface.version(),
                    id_version
                ),
            )
            .await;
            return;
        }

        match id_interface.as_str() {
            WlShm::NAME => {
                let _ = self
                    .stream
                    .write(
                        &WlShm { object_id: id }
                            .format(WlShmFormat::Argb8888)
                            .to_raw(),
                    )
                    .await;
            }
            WlOutput::NAME => {
                let geometry = self.shared_state.get_box();
                // FIXME: Make difference between physical and logical size
                let mut data = Vec::new();
                let wl_output = WlOutput { object_id: id };
                data.extend(
                    wl_output
                        .geometry(
                            geometry.x as i32,
                            geometry.y as i32,
                            geometry.w as i32,
                            geometry.h as i32,
                            WlOutputSubpixel::Unknown,
                            "Not your buisness".to_owned(),
                            "Not your buisness".to_owned(),
                            WlOutputTransform::Normal,
                        )
                        .to_raw(),
                );
                data.extend(wl_output.done().to_raw());
                let _ = self.stream.write(&data).await;
            }
            _ => {}
        }

        self.add_boxed_object(interface);
    }
}

#[async_trait]
impl WlCallbackListener for Client {}

#[allow(unused_variables)]
#[async_trait]
impl WlCompositorListener for Client {
    async fn create_surface(&mut self, compositor: WlCompositor, surface: WlSurface) {
        let surface = ImpWlSurface {
            inner: surface,
            buffer: None,
        };
        self.add_object(surface.inner);
        self.wl_state_mut().surfaces.insert(surface.inner, surface);
    }

    async fn create_region(&mut self, compositor: WlCompositor, region: WlRegion) {
        todo!()
    }
}

#[allow(unused_variables)]
#[async_trait]
impl WlShmPoolListener for Client {
    async fn create_buffer(
        &mut self,
        pool: WlShmPool,
        buffer: WlBuffer,
        offset: i32,
        width: i32,
        height: i32,
        stride: i32,
        format: WlShmFormat,
    ) {
        let Some(pool) = self.wl_state_mut().shm_pools.get_mut(&pool) else {
            error!(
                "wl_shm_pool#{}::create_buffer: pool not found",
                pool.object_id
            );
            return;
        };
        if stride != width * 4 {
            error!(
                "wl_shm_pool#{}::create_buffer: stride and width don't match: {} != {}",
                pool.inner.object_id, stride, width
            );
            return;
        }
        let buffer = ImpWlBuffer {
            inner: buffer,
            // buf: WaylandBuffer::new(width as usize, height as usize, stride as usize),
            width: width as usize,
            height: height as usize,
        };
        pool.buffers.push(buffer.inner);
        self.add_object(buffer.inner);
        self.wl_state_mut().buffers.insert(buffer.inner, buffer);
    }

    async fn destroy(&mut self, pool: WlShmPool) {
        self.remove_object(pool.object_id);
        let _ = self
            .stream
            .write(&DISPLAY_OBJECT.delete_id(pool.object_id).to_raw())
            .await;
    }

    async fn resize(&mut self, pool: WlShmPool, size: i32) {
        todo!()
    }
}

#[allow(unused_variables)]
#[async_trait]
impl WlShmListener for Client {
    async fn create_pool(&mut self, shm: WlShm, pool: WlShmPool, fd: OwnedFd, size: i32) {
        use std::os::fd::{FromRawFd as _, IntoRawFd as _};
        let file = unsafe { std::fs::File::from_raw_fd(fd.into_raw_fd()) };
        let mmap = match unsafe { memmap::MmapOptions::new().len(size as usize).map_mut(&file) } {
            Ok(mmap) => mmap,
            Err(err) => {
                self.error(
                    pool.object_id,
                    WlShmError::InvalidFd as u32,
                    err.to_string(),
                )
                .await;
                return;
            }
        };
        let shm = ImpWlShmPool {
            inner: pool,
            mmap,
            buffers: Vec::new(),
        };
        self.imp_proto_states
            .wayland
            .shm_pools
            .insert(shm.inner, shm);
        self.add_object(pool);
    }

    async fn release(&mut self, shm: WlShm) {
        todo!()
    }
}

#[allow(unused_variables)]
#[async_trait]
impl WlBufferListener for Client {
    async fn destroy(&mut self, buffer: WlBuffer) {
        // TODO: destroy buffers
        self.wl_state_mut().buffers.remove(&buffer);
    }
}

#[allow(unused_variables)]
#[async_trait]
impl WlSurfaceListener for Client {
    async fn destroy(&mut self, surface: WlSurface) {
        todo!()
    }

    async fn attach(&mut self, surface: WlSurface, buffer: Option<WlBuffer>, x: i32, y: i32) {
        let Some(surface) = self.wl_state_mut().surfaces.get_mut(&surface) else {
            return;
        };
        surface.buffer = buffer;
        dbg!(&surface.buffer);
    }

    async fn damage(&mut self, surface: WlSurface, x: i32, y: i32, width: i32, height: i32) {
        todo!()
    }

    async fn frame(&mut self, surface: WlSurface, callback: WlCallback) {
        todo!()
    }

    async fn set_opaque_region(&mut self, surface: WlSurface, region: Option<WlRegion>) {
        todo!()
    }

    async fn set_input_region(&mut self, surface: WlSurface, region: Option<WlRegion>) {
        todo!()
    }

    async fn commit(&mut self, surface: WlSurface) {
        let object_id = surface.object_id;
        let Some(surface) = self.wl_state().surfaces.get(&surface) else {
            error!("wl_surface#{object_id}::commit: surface not found",);
            return;
        };

        let Some(buffer) = surface.buffer else {
            warn!("wl_surface#{object_id}::commit: surface has no buffer",);
            return;
        };

        let Some(pool) = self
            .wl_state()
            .shm_pools
            .iter()
            .find_map(|(_, pool)| pool.buffers.contains(&buffer).then_some(pool))
        else {
            error!("wl_surface#{object_id}::commit: No shm pool for buffer",);
            return;
        };
        let pool = self.wl_state().shm_pools.get(&pool.inner).unwrap();
        // TODO: Do I need to copy the data?
        let data = pool
            .mmap
            .chunks_exact(4)
            .map(|c| u32::from_ne_bytes([c[0], c[1], c[2], c[3]]))
            .collect::<Vec<_>>();

        let buffer = self.wl_state_mut().buffers.get_mut(&buffer).unwrap();
        // if buffer.buf.data.len() != data.len() {
        let elen = buffer.width * buffer.height;
        if elen != data.len() {
            error!(
                "wl_surface#{object_id}::commit: buffer size {} != shmem size {}",
                elen,
                data.len()
            );
            return;
        }
        // buffer.buf.data = data;
        // TODO: Do I need to copy the buffer?
        // let buf = buffer.buf.clone();
        let buf = WaylandBuffer {
            data,
            stride: buffer.width,
        };
        self.shared_state.render(buf);
    }

    async fn set_buffer_transform(&mut self, surface: WlSurface, transform: WlOutputTransform) {
        todo!()
    }

    async fn set_buffer_scale(&mut self, surface: WlSurface, scale: i32) {
        todo!()
    }

    async fn damage_buffer(&mut self, surface: WlSurface, x: i32, y: i32, width: i32, height: i32) {
        todo!()
    }

    async fn offset(&mut self, surface: WlSurface, x: i32, y: i32) {
        todo!()
    }
}

#[allow(unused_variables)]
#[async_trait]
impl WlOutputListener for Client {
    async fn release(&mut self, output: WlOutput) {
        todo!()
    }
}
