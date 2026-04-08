use crate::DISPLAY_OBJECT;
use crate::PocoWlClient;
use memmap::MmapMut;
use pocowl_protocols::WaylandProtocol;
use pocowl_protocols::wayland::*;
use pocowl_protocols::xdg_shell::XdgWmBase;
use pocowl_wlbuffer::WaylandBuffer;
use std::collections::HashMap;
use std::os::fd::OwnedFd;
use std::rc::Rc;
use tokio::io::AsyncWriteExt as _;

const SUPPORTED_INTERFACE_FACTORIES: [fn(u32) -> Rc<dyn WaylandProtocol<PocoWlClient>>; 4] = [
    |id| Rc::new(WlCompositor { object_id: id }),
    |id| Rc::new(WlShm { object_id: id }),
    |id| Rc::new(XdgWmBase { object_id: id }),
    |id| Rc::new(WlOutput { object_id: id }),
];

pub struct PocoWlState {
    buffers: HashMap<WlBuffer, WaylandBuffer>,
    shms: HashMap<WlShmPool, MmapMut>,
    pool_buffers: HashMap<WlShmPool, Vec<WlBuffer>>,
    surface_buffers: HashMap<WlSurface, WlBuffer>,
}
impl PocoWlState {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
            shms: HashMap::new(),
            pool_buffers: HashMap::new(),
            surface_buffers: HashMap::new(),
        }
    }
}

impl WlDisplayListener for PocoWlClient {
    async fn sync(&mut self, object: WlDisplay, callback: WlCallback) {
        _ = object;
        let mut data = Vec::new();
        // data.extend(WlDisplay::delete_id(object, callback.object_id).to_raw());
        data.extend(WlCallback::done(callback, Default::default()).to_raw());
        let _ = self.client.stream.write(&data).await;
    }

    async fn get_registry(&mut self, object: WlDisplay, registry: WlRegistry) {
        _ = object;
        self.objects.insert(registry.object_id, Rc::new(registry));
        let mut data = Vec::new();
        for (name, interface_factory) in SUPPORTED_INTERFACE_FACTORIES.iter().enumerate() {
            let interface = (interface_factory)(registry.object_id);
            data.extend(
                WlRegistry::global(
                    registry,
                    name as u32,
                    interface.name().to_owned(),
                    interface.version(),
                )
                .to_raw(),
            );
        }
        let _ = self.client.stream.write(&data).await;
    }
}

impl WlRegistryListener for PocoWlClient {
    async fn bind(
        &mut self,
        object: WlRegistry,
        name: u32,
        id_interface: String,
        id_version: u32,
        id: u32,
    ) {
        let Some(interface_factory) = SUPPORTED_INTERFACE_FACTORIES.get(name as usize) else {
            let _ = self
                .client
                .stream
                .write(
                    &WlDisplay::error(
                        DISPLAY_OBJECT,
                        object.object_id,
                        0,
                        format!("Invalid interface name: {}", name,),
                    )
                    .to_raw(),
                )
                .await;
            // TODO: Close socket
            return;
        };
        let interface = (interface_factory)(id);
        if id_interface != interface.name() {
            let _ = self
                .client
                .stream
                .write(
                    &WlDisplay::error(
                        DISPLAY_OBJECT,
                        object.object_id,
                        0,
                        format!(
                            "Expected interface {}, found {}",
                            interface.name(),
                            id_interface
                        ),
                    )
                    .to_raw(),
                )
                .await;
            // TODO: Close socket
            return;
        }
        if id_version > interface.version() {
            let _ = self
                .client
                .stream
                .write(
                    &WlDisplay::error(
                        DISPLAY_OBJECT,
                        object.object_id,
                        0,
                        format!(
                            "Expected interface version < {}, found {}",
                            interface.version(),
                            id_version
                        ),
                    )
                    .to_raw(),
                )
                .await;
            // TODO: Close socket
            return;
        }

        match id_interface.as_str() {
            WlShm::NAME => {
                let _ = self
                    .client
                    .stream
                    .write(&WlShm::format(WlShm { object_id: id }, WlShmFormat::Argb8888).to_raw())
                    .await;
            }
            WlOutput::NAME => {
                let (x, y, w, h) = self.backend_sender.get_box().await;
                // FIXME: Make difference between physical and logical size
                let mut data = Vec::new();
                let wl_output = WlOutput { object_id: id };
                data.extend(
                    WlOutput::geometry(
                        wl_output,
                        x as i32,
                        y as i32,
                        w as i32,
                        h as i32,
                        WlOutputSubpixel::Unknown,
                        "Not your buisness".to_owned(),
                        "Not your buisness".to_owned(),
                        WlOutputTransform::Normal,
                    )
                    .to_raw(),
                );
                data.extend(WlOutput::done(wl_output).to_raw());
                let _ = self.client.stream.write(&data).await;
            }
            _ => {}
        }

        self.objects.insert(id, interface);
    }
}

impl WlCallbackListener for PocoWlClient {}

#[allow(unused_variables)]
impl WlCompositorListener for PocoWlClient {
    async fn create_surface(&mut self, object: WlCompositor, id: WlSurface) {
        self.objects.insert(id.object_id, Rc::new(id));
    }

    async fn create_region(&mut self, object: WlCompositor, id: WlRegion) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShmPoolListener for PocoWlClient {
    async fn create_buffer(
        &mut self,
        object: WlShmPool,
        id: WlBuffer,
        offset: i32,
        width: i32,
        height: i32,
        stride: i32,
        format: WlShmFormat,
    ) {
        self.wl_state.buffers.insert(
            id,
            WaylandBuffer::new(width as usize, height as usize, stride as usize),
        );
        self.wl_state
            .pool_buffers
            .entry(object)
            .or_default()
            .push(id);
        self.objects.insert(id.object_id, Rc::new(id));
    }

    async fn destroy(&mut self, object: WlShmPool) {
        self.objects.remove(&object.object_id);
        let _ = self
            .client
            .stream
            .write(&WlDisplay::delete_id(DISPLAY_OBJECT, object.object_id).to_raw())
            .await;
    }

    async fn resize(&mut self, object: WlShmPool, size: i32) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShmListener for PocoWlClient {
    async fn create_pool(&mut self, object: WlShm, id: WlShmPool, fd: OwnedFd, size: i32) {
        use std::os::fd::{FromRawFd as _, IntoRawFd as _};
        let file = unsafe { std::fs::File::from_raw_fd(fd.into_raw_fd()) };
        let mmap = match unsafe { memmap::MmapOptions::new().len(size as usize).map_mut(&file) } {
            Ok(mmap) => mmap,
            Err(err) => {
                let _ = self
                    .client
                    .stream
                    .write(
                        &WlDisplay::error(
                            DISPLAY_OBJECT,
                            id.object_id,
                            WlShmError::InvalidFd as u32,
                            err.to_string(),
                        )
                        .to_raw(),
                    )
                    .await;
                return;
            }
        };
        self.wl_state.shms.insert(id, mmap);

        self.objects.insert(id.object_id, Rc::new(id));
    }

    async fn release(&mut self, object: WlShm) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlBufferListener for PocoWlClient {
    async fn destroy(&mut self, object: WlBuffer) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataOfferListener for PocoWlClient {
    async fn accept(&mut self, object: WlDataOffer, serial: u32, mime_type: Option<String>) {
        todo!()
    }

    async fn receive(&mut self, object: WlDataOffer, mime_type: String, fd: OwnedFd) {
        todo!()
    }

    async fn destroy(&mut self, object: WlDataOffer) {
        todo!()
    }

    async fn finish(&mut self, object: WlDataOffer) {
        todo!()
    }

    async fn set_actions(
        &mut self,
        object: WlDataOffer,
        dnd_actions: WlDataDeviceManagerDndAction,
        preferred_action: WlDataDeviceManagerDndAction,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataSourceListener for PocoWlClient {
    async fn offer(&mut self, object: WlDataSource, mime_type: String) {
        todo!()
    }

    async fn destroy(&mut self, object: WlDataSource) {
        todo!()
    }

    async fn set_actions(
        &mut self,
        object: WlDataSource,
        dnd_actions: WlDataDeviceManagerDndAction,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataDeviceListener for PocoWlClient {
    async fn start_drag(
        &mut self,
        object: WlDataDevice,
        source: Option<WlDataSource>,
        origin: WlSurface,
        icon: Option<WlSurface>,
        serial: u32,
    ) {
        todo!()
    }

    async fn set_selection(
        &mut self,
        object: WlDataDevice,
        source: Option<WlDataSource>,
        serial: u32,
    ) {
        todo!()
    }

    async fn release(&mut self, object: WlDataDevice) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataDeviceManagerListener for PocoWlClient {
    async fn create_data_source(&mut self, object: WlDataDeviceManager, id: WlDataSource) {
        todo!()
    }

    async fn get_data_device(
        &mut self,
        object: WlDataDeviceManager,
        id: WlDataDevice,
        seat: WlSeat,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShellListener for PocoWlClient {
    async fn get_shell_surface(&mut self, object: WlShell, id: WlShellSurface, surface: WlSurface) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShellSurfaceListener for PocoWlClient {
    async fn pong(&mut self, object: WlShellSurface, serial: u32) {
        todo!()
    }

    async fn r#move(&mut self, object: WlShellSurface, seat: WlSeat, serial: u32) {
        todo!()
    }

    async fn resize(
        &mut self,
        object: WlShellSurface,
        seat: WlSeat,
        serial: u32,
        edges: WlShellSurfaceResize,
    ) {
        todo!()
    }

    async fn set_toplevel(&mut self, object: WlShellSurface) {
        todo!()
    }

    async fn set_transient(
        &mut self,
        object: WlShellSurface,
        parent: WlSurface,
        x: i32,
        y: i32,
        flags: WlShellSurfaceTransient,
    ) {
        todo!()
    }

    async fn set_fullscreen(
        &mut self,
        object: WlShellSurface,
        method: WlShellSurfaceFullscreenMethod,
        framerate: u32,
        output: Option<WlOutput>,
    ) {
        todo!()
    }

    async fn set_popup(
        &mut self,
        object: WlShellSurface,
        seat: WlSeat,
        serial: u32,
        parent: WlSurface,
        x: i32,
        y: i32,
        flags: WlShellSurfaceTransient,
    ) {
        todo!()
    }

    async fn set_maximized(&mut self, object: WlShellSurface, output: Option<WlOutput>) {
        todo!()
    }

    async fn set_title(&mut self, object: WlShellSurface, title: String) {
        todo!()
    }

    async fn set_class(&mut self, object: WlShellSurface, class_: String) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSurfaceListener for PocoWlClient {
    async fn destroy(&mut self, object: WlSurface) {
        todo!()
    }

    async fn attach(&mut self, object: WlSurface, buffer: Option<WlBuffer>, x: i32, y: i32) {
        _ = x;
        _ = y;
        if let Some(buffer) = buffer {
            self.wl_state.surface_buffers.insert(object, buffer);
        } else {
            self.wl_state.surface_buffers.remove(&object);
        }
    }

    async fn damage(&mut self, object: WlSurface, x: i32, y: i32, width: i32, height: i32) {
        todo!()
    }

    async fn frame(&mut self, object: WlSurface, callback: WlCallback) {
        todo!()
    }

    async fn set_opaque_region(&mut self, object: WlSurface, region: Option<WlRegion>) {
        todo!()
    }

    async fn set_input_region(&mut self, object: WlSurface, region: Option<WlRegion>) {
        todo!()
    }

    async fn commit(&mut self, object: WlSurface) {
        let buffer = self
            .wl_state
            .surface_buffers
            .get_mut(&object)
            .and_then(|wl_buffer| self.wl_state.buffers.get_mut(wl_buffer));
        let shmem = self
            .wl_state
            .surface_buffers
            .get(&object)
            .and_then(|wl_buffer| {
                self.wl_state
                    .pool_buffers
                    .iter()
                    .find_map(|(pool, buffers)| buffers.contains(wl_buffer).then_some(pool))
            })
            .and_then(|pool| self.wl_state.shms.get(pool));
        let Some((buffer, shmem)) = Option::zip(buffer, shmem) else {
            return;
        };
        if buffer.data.len() != shmem.len() {
            eprintln!(
                "Commit: buffer size {} != shmem size {}",
                buffer.data.len(),
                shmem.len()
            );
            return;
        }
        // TODO: is it necessary to copy the data?
        buffer.data = shmem.to_vec();
        self.backend_sender.draw(0, 0, buffer.clone()).await;
        // TODO: swap buffers?
    }

    async fn set_buffer_transform(&mut self, object: WlSurface, transform: WlOutputTransform) {
        todo!()
    }

    async fn set_buffer_scale(&mut self, object: WlSurface, scale: i32) {
        todo!()
    }

    async fn damage_buffer(&mut self, object: WlSurface, x: i32, y: i32, width: i32, height: i32) {
        todo!()
    }

    async fn offset(&mut self, object: WlSurface, x: i32, y: i32) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSeatListener for PocoWlClient {
    async fn get_pointer(&mut self, object: WlSeat, id: WlPointer) {
        todo!()
    }

    async fn get_keyboard(&mut self, object: WlSeat, id: WlKeyboard) {
        todo!()
    }

    async fn get_touch(&mut self, object: WlSeat, id: WlTouch) {
        todo!()
    }

    async fn release(&mut self, object: WlSeat) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlPointerListener for PocoWlClient {
    async fn set_cursor(
        &mut self,
        object: WlPointer,
        serial: u32,
        surface: Option<WlSurface>,
        hotspot_x: i32,
        hotspot_y: i32,
    ) {
        todo!()
    }

    async fn release(&mut self, object: WlPointer) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlKeyboardListener for PocoWlClient {
    async fn release(&mut self, object: WlKeyboard) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlTouchListener for PocoWlClient {
    async fn release(&mut self, object: WlTouch) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlOutputListener for PocoWlClient {
    async fn release(&mut self, object: WlOutput) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlRegionListener for PocoWlClient {
    async fn destroy(&mut self, object: WlRegion) {
        todo!()
    }

    async fn add(&mut self, object: WlRegion, x: i32, y: i32, width: i32, height: i32) {
        todo!()
    }

    async fn subtract(&mut self, object: WlRegion, x: i32, y: i32, width: i32, height: i32) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSubcompositorListener for PocoWlClient {
    async fn destroy(&mut self, object: WlSubcompositor) {
        todo!()
    }

    async fn get_subsurface(
        &mut self,
        object: WlSubcompositor,
        id: WlSubsurface,
        surface: WlSurface,
        parent: WlSurface,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSubsurfaceListener for PocoWlClient {
    async fn destroy(&mut self, object: WlSubsurface) {
        todo!()
    }

    async fn set_position(&mut self, object: WlSubsurface, x: i32, y: i32) {
        todo!()
    }

    async fn place_above(&mut self, object: WlSubsurface, sibling: WlSurface) {
        todo!()
    }

    async fn place_below(&mut self, object: WlSubsurface, sibling: WlSurface) {
        todo!()
    }

    async fn set_sync(&mut self, object: WlSubsurface) {
        todo!()
    }

    async fn set_desync(&mut self, object: WlSubsurface) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlFixesListener for PocoWlClient {
    async fn destroy(&mut self, object: WlFixes) {
        todo!()
    }

    async fn destroy_registry(&mut self, object: WlFixes, registry: WlRegistry) {
        todo!()
    }
}
