use crate::DISPLAY_OBJECT;
use crate::PocoWlClient;
use pocowl_protocols::WaylandProtocol;
use pocowl_protocols::wayland::*;
use pocowl_protocols::xdg_shell::XdgWmBase;
use pocowl_wlclient::WaylandClient;
use std::rc::Rc;

const SUPPORTED_INTERFACE_FACTORIES: [fn(u32) -> Rc<dyn WaylandProtocol<PocoWlClient>>; 3] = [
    |id| Rc::new(WlCompositor { object_id: id }),
    |id| Rc::new(WlShm { object_id: id }),
    |id| Rc::new(XdgWmBase { object_id: id }),
];

impl WlDisplayListener for PocoWlClient {
    fn sync(&mut self, object: WlDisplay, callback: WlCallback, client: &mut WaylandClient) {
        _ = object;
        let mut data = Vec::new();
        // data.extend(WlDisplay::delete_id(object, callback.object_id).to_raw());
        data.extend(WlCallback::done(callback, Default::default()).to_raw());
        client.stream.try_write(&data).unwrap();
    }

    fn get_registry(
        &mut self,
        object: WlDisplay,
        registry: WlRegistry,
        client: &mut WaylandClient,
    ) {
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
        client.stream.try_write(&data).unwrap();
    }
}

impl WlRegistryListener for PocoWlClient {
    fn bind(
        &mut self,
        object: WlRegistry,
        name: u32,
        id_interface: String,
        id_version: u32,
        id: u32,
        client: &mut WaylandClient,
    ) {
        // let interface = get_interface(name);
        let Some(interface_factory) = SUPPORTED_INTERFACE_FACTORIES.get(name as usize) else {
            client
                .stream
                .try_write(
                    &WlDisplay::error(
                        DISPLAY_OBJECT,
                        object.object_id,
                        0,
                        format!("Invalid interface name: {}", name,),
                    )
                    .to_raw(),
                )
                .unwrap();
            // TODO: Close socket
            return;
        };
        let interface = (interface_factory)(id);
        if id_interface != interface.name() {
            client
                .stream
                .try_write(
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
                .unwrap();
            // TODO: Close socket
            return;
        }
        if id_version > interface.version() {
            client
                .stream
                .try_write(
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
                .unwrap();
            // TODO: Close socket
            return;
        }
        self.objects.insert(id, interface);
        // if let Some(registries) = self.clients_registries.get(&client.id) {
        //     for registry in registries {
        //         send_supports_interfaces(*registry, client);
        //     }
        // }
    }
}

impl WlCallbackListener for PocoWlClient {}

#[allow(unused_variables)]
impl WlCompositorListener for PocoWlClient {
    fn create_surface(&mut self, object: WlCompositor, id: WlSurface, client: &mut WaylandClient) {
        todo!()
    }

    fn create_region(&mut self, object: WlCompositor, id: WlRegion, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShmPoolListener for PocoWlClient {
    fn create_buffer(
        &mut self,
        object: WlShmPool,
        id: WlBuffer,
        offset: i32,
        width: i32,
        height: i32,
        stride: i32,
        format: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn destroy(&mut self, object: WlShmPool, client: &mut WaylandClient) {
        todo!()
    }

    fn resize(&mut self, object: WlShmPool, size: i32, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShmListener for PocoWlClient {
    fn create_pool(
        &mut self,
        object: WlShm,
        id: WlShmPool,
        fd: (),
        size: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn release(&mut self, object: WlShm, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlBufferListener for PocoWlClient {
    fn destroy(&mut self, object: WlBuffer, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataOfferListener for PocoWlClient {
    fn accept(
        &mut self,
        object: WlDataOffer,
        serial: u32,
        mime_type: Option<String>,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn receive(
        &mut self,
        object: WlDataOffer,
        mime_type: String,
        fd: (),
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn destroy(&mut self, object: WlDataOffer, client: &mut WaylandClient) {
        todo!()
    }

    fn finish(&mut self, object: WlDataOffer, client: &mut WaylandClient) {
        todo!()
    }

    fn set_actions(
        &mut self,
        object: WlDataOffer,
        dnd_actions: u32,
        preferred_action: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataSourceListener for PocoWlClient {
    fn offer(&mut self, object: WlDataSource, mime_type: String, client: &mut WaylandClient) {
        todo!()
    }

    fn destroy(&mut self, object: WlDataSource, client: &mut WaylandClient) {
        todo!()
    }

    fn set_actions(&mut self, object: WlDataSource, dnd_actions: u32, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataDeviceListener for PocoWlClient {
    fn start_drag(
        &mut self,
        object: WlDataDevice,
        source: Option<WlDataSource>,
        origin: WlSurface,
        icon: Option<WlSurface>,
        serial: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_selection(
        &mut self,
        object: WlDataDevice,
        source: Option<WlDataSource>,
        serial: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn release(&mut self, object: WlDataDevice, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlDataDeviceManagerListener for PocoWlClient {
    fn create_data_source(
        &mut self,
        object: WlDataDeviceManager,
        id: WlDataSource,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn get_data_device(
        &mut self,
        object: WlDataDeviceManager,
        id: WlDataDevice,
        seat: WlSeat,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShellListener for PocoWlClient {
    fn get_shell_surface(
        &mut self,
        object: WlShell,
        id: WlShellSurface,
        surface: WlSurface,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlShellSurfaceListener for PocoWlClient {
    fn pong(&mut self, object: WlShellSurface, serial: u32, client: &mut WaylandClient) {
        todo!()
    }

    fn r#move(
        &mut self,
        object: WlShellSurface,
        seat: WlSeat,
        serial: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn resize(
        &mut self,
        object: WlShellSurface,
        seat: WlSeat,
        serial: u32,
        edges: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_toplevel(&mut self, object: WlShellSurface, client: &mut WaylandClient) {
        todo!()
    }

    fn set_transient(
        &mut self,
        object: WlShellSurface,
        parent: WlSurface,
        x: i32,
        y: i32,
        flags: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_fullscreen(
        &mut self,
        object: WlShellSurface,
        method: u32,
        framerate: u32,
        output: Option<WlOutput>,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_popup(
        &mut self,
        object: WlShellSurface,
        seat: WlSeat,
        serial: u32,
        parent: WlSurface,
        x: i32,
        y: i32,
        flags: u32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_maximized(
        &mut self,
        object: WlShellSurface,
        output: Option<WlOutput>,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_title(&mut self, object: WlShellSurface, title: String, client: &mut WaylandClient) {
        todo!()
    }

    fn set_class(&mut self, object: WlShellSurface, class_: String, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSurfaceListener for PocoWlClient {
    fn destroy(&mut self, object: WlSurface, client: &mut WaylandClient) {
        todo!()
    }

    fn attach(
        &mut self,
        object: WlSurface,
        buffer: Option<WlBuffer>,
        x: i32,
        y: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn damage(
        &mut self,
        object: WlSurface,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn frame(&mut self, object: WlSurface, callback: WlCallback, client: &mut WaylandClient) {
        todo!()
    }

    fn set_opaque_region(
        &mut self,
        object: WlSurface,
        region: Option<WlRegion>,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_input_region(
        &mut self,
        object: WlSurface,
        region: Option<WlRegion>,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn commit(&mut self, object: WlSurface, client: &mut WaylandClient) {
        todo!()
    }

    fn set_buffer_transform(
        &mut self,
        object: WlSurface,
        transform: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_buffer_scale(&mut self, object: WlSurface, scale: i32, client: &mut WaylandClient) {
        todo!()
    }

    fn damage_buffer(
        &mut self,
        object: WlSurface,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn offset(&mut self, object: WlSurface, x: i32, y: i32, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSeatListener for PocoWlClient {
    fn get_pointer(&mut self, object: WlSeat, id: WlPointer, client: &mut WaylandClient) {
        todo!()
    }

    fn get_keyboard(&mut self, object: WlSeat, id: WlKeyboard, client: &mut WaylandClient) {
        todo!()
    }

    fn get_touch(&mut self, object: WlSeat, id: WlTouch, client: &mut WaylandClient) {
        todo!()
    }

    fn release(&mut self, object: WlSeat, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlPointerListener for PocoWlClient {
    fn set_cursor(
        &mut self,
        object: WlPointer,
        serial: u32,
        surface: Option<WlSurface>,
        hotspot_x: i32,
        hotspot_y: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn release(&mut self, object: WlPointer, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlKeyboardListener for PocoWlClient {
    fn release(&mut self, object: WlKeyboard, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlTouchListener for PocoWlClient {
    fn release(&mut self, object: WlTouch, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlOutputListener for PocoWlClient {
    fn release(&mut self, object: WlOutput, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlRegionListener for PocoWlClient {
    fn destroy(&mut self, object: WlRegion, client: &mut WaylandClient) {
        todo!()
    }

    fn add(
        &mut self,
        object: WlRegion,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn subtract(
        &mut self,
        object: WlRegion,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSubcompositorListener for PocoWlClient {
    fn destroy(&mut self, object: WlSubcompositor, client: &mut WaylandClient) {
        todo!()
    }

    fn get_subsurface(
        &mut self,
        object: WlSubcompositor,
        id: WlSubsurface,
        surface: WlSurface,
        parent: WlSurface,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlSubsurfaceListener for PocoWlClient {
    fn destroy(&mut self, object: WlSubsurface, client: &mut WaylandClient) {
        todo!()
    }

    fn set_position(&mut self, object: WlSubsurface, x: i32, y: i32, client: &mut WaylandClient) {
        todo!()
    }

    fn place_above(
        &mut self,
        object: WlSubsurface,
        sibling: WlSurface,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn place_below(
        &mut self,
        object: WlSubsurface,
        sibling: WlSurface,
        client: &mut WaylandClient,
    ) {
        todo!()
    }

    fn set_sync(&mut self, object: WlSubsurface, client: &mut WaylandClient) {
        todo!()
    }

    fn set_desync(&mut self, object: WlSubsurface, client: &mut WaylandClient) {
        todo!()
    }
}

#[allow(unused_variables)]
impl WlFixesListener for PocoWlClient {
    fn destroy(&mut self, object: WlFixes, client: &mut WaylandClient) {
        todo!()
    }

    fn destroy_registry(
        &mut self,
        object: WlFixes,
        registry: WlRegistry,
        client: &mut WaylandClient,
    ) {
        todo!()
    }
}
