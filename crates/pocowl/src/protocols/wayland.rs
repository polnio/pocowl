use crate::PocoWlClient;
use pocowl_protocols::wayland::{
    WlCallback, WlCallbackListener, WlCompositor, WlCompositorListener, WlDisplay,
    WlRegistryListener, WlShm, WlShmListener, WlShmPoolListener,
};
use pocowl_protocols::wayland::{WlDisplayListener, WlRegistry};
use pocowl_protocols::xdg_shell::XdgWmBase;
use pocowl_wlclient::WaylandClient;
use std::rc::Rc;

macro_rules! supports_interfaces {
    ($($interface:tt),*) => {
        fn send_supports_interfaces(registry: u32, client: &mut WaylandClient) {
            let mut data = Vec::new();
            let mut i = 1;
            $(send_supports_interface!(data, registry, i, $interface);)*
            _ = i;
            println!("S -> C: {data:?}");
            client.stream.try_write(&data).unwrap();
        }
        fn get_interface(name: u32) -> Rc<dyn pocowl_protocols::WaylandProtocol<crate::PocoWlClient>> {
            let mut i = 1;
            $(
                if name == i {
                    return Rc::new($interface);
                }
                i += 1;
            )*
            panic!("No interface found for {}", name);
        }
    }
}

macro_rules! send_supports_interface {
    ($data:expr, $object:expr, $i:expr, $interface:tt) => {
        $data.extend(
            WlRegistry::global(
                $object,
                $i,
                $interface::NAME.to_owned(),
                $interface::VERSION,
            )
            .to_raw(),
        );
        $i += 1;
    };
}

supports_interfaces!(WlCompositor, WlShm, XdgWmBase);

impl WlDisplayListener for PocoWlClient {
    fn sync(&mut self, object_id: u32, callback: u32, client: &mut WaylandClient) {
        _ = object_id;
        let mut data = Vec::new();
        // data.extend(WlDisplay::delete_id(object_id, callback).to_raw());
        data.extend(WlCallback::done(callback, Default::default()).to_raw());
        client.stream.try_write(&data).unwrap();
    }

    fn get_registry(&mut self, object_id: u32, registry: u32, client: &mut WaylandClient) {
        _ = object_id;
        self.objects.insert(registry, Rc::new(WlRegistry));
        send_supports_interfaces(registry, client);
    }
}

impl WlRegistryListener for PocoWlClient {
    fn bind(
        &mut self,
        object_id: u32,
        name: u32,
        id_interface: String,
        id_version: u32,
        id: u32,
        client: &mut WaylandClient,
    ) {
        _ = client;
        let interface = get_interface(name);
        if id_interface != interface.name() {
            client
                .stream
                .try_write(
                    &WlDisplay::error(
                        1,
                        object_id,
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
                        1,
                        object_id,
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
        println!(
            "Registry bind {}: {} - {} - {}",
            id, name, id_interface, id_version
        );
        self.objects.insert(id, interface);
        // if let Some(registries) = self.clients_registries.get(&client.id) {
        //     for registry in registries {
        //         send_supports_interfaces(*registry, client);
        //     }
        // }
    }
}

impl WlCallbackListener for PocoWlClient {}

impl WlCompositorListener for PocoWlClient {
    fn create_surface(&mut self, object_id: u32, id: u32, client: &mut WaylandClient) {
        _ = object_id;
        _ = id;
        _ = client;
        todo!()
    }

    fn create_region(&mut self, object_id: u32, id: u32, client: &mut WaylandClient) {
        _ = object_id;
        _ = id;
        _ = client;
        todo!()
    }
}

impl WlShmPoolListener for PocoWlClient {
    fn create_buffer(
        &mut self,
        object_id: u32,
        id: u32,
        offset: i32,
        width: i32,
        height: i32,
        stride: i32,
        format: u32,
        client: &mut WaylandClient,
    ) {
        _ = object_id;
        _ = id;
        _ = offset;
        _ = width;
        _ = height;
        _ = stride;
        _ = format;
        _ = client;
        todo!()
    }

    fn destroy(&mut self, object_id: u32, client: &mut WaylandClient) {
        _ = object_id;
        _ = client;
        todo!()
    }

    fn resize(&mut self, object_id: u32, size: i32, client: &mut WaylandClient) {
        _ = object_id;
        _ = size;
        _ = client;
        todo!()
    }
}

impl WlShmListener for PocoWlClient {
    fn create_pool(
        &mut self,
        object_id: u32,
        id: u32,
        fd: (),
        size: i32,
        client: &mut WaylandClient,
    ) {
        _ = object_id;
        _ = id;
        _ = fd;
        _ = size;
        _ = client;
        todo!()
    }

    fn release(&mut self, object_id: u32, client: &mut WaylandClient) {
        _ = object_id;
        _ = client;
        todo!()
    }
}
