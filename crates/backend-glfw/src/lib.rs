use crossbeam::channel::Receiver;
use glfw::Context as _;
use ouroboros::self_referencing;
use pocowl_backend::{Backend, Message};
use std::num::NonZeroU32;

#[self_referencing]
pub struct GlfwBackendWindow {
    window: glfw::PWindow,
    #[borrows(window)]
    #[covariant]
    surface: softbuffer::Surface<&'this glfw::PWindow, &'this glfw::PWindow>,
}
pub struct GlfwBackend;
impl GlfwBackend {
    pub fn new() -> Self {
        Self
    }

    fn run(&mut self, rx: Receiver<Message>) {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init GLFW");
        let (mut window, events) = glfw
            .create_window(800, 600, "GLFW", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");
        window.set_size_polling(true);
        window.make_current();

        let mut window = GlfwBackendWindow::new(window, |window| {
            let sbctx =
                softbuffer::Context::new(window).expect("Failed to create softbuffer context");
            let surface =
                softbuffer::Surface::new(&sbctx, window).expect("Failed to create surface");
            surface
        });

        window.with_surface_mut(|surface| {
            surface
                .resize(NonZeroU32::new(800).unwrap(), NonZeroU32::new(600).unwrap())
                .unwrap();
            surface.buffer_mut().unwrap().present().unwrap();
        });

        let window = std::sync::Arc::new(std::sync::Mutex::new(window));

        let (events_tx, events_rx) = crossbeam::channel::unbounded::<Vec<glfw::WindowEvent>>();

        std::thread::spawn({
            let window = window.clone();
            move || {
                loop {
                    let c = crossbeam::select! {
                        recv(events_rx) -> events => Self::handle_events(
                            &events.unwrap_or_else(|_| Vec::new()),
                            &mut window.lock().unwrap()
                        ),
                        recv(rx) -> message => Self::handle_message(
                            message.unwrap_or(Message::Quit),
                            &mut window.lock().unwrap()
                        ),
                    };
                    if !c {
                        break;
                    }
                }
            }
        });

        while !window.lock().unwrap().borrow_window().should_close() {
            glfw.wait_events();

            let events: Vec<_> = glfw::flush_messages(&events).map(|(_, e)| e).collect();
            if events_tx.send(events).is_err() {
                break;
            };
        }
    }

    fn handle_events(events: &[glfw::WindowEvent], window: &mut GlfwBackendWindow) -> bool {
        for event in events {
            match event {
                glfw::WindowEvent::Size(w, h) => window.with_surface_mut(|surface| {
                    surface
                        .resize(
                            NonZeroU32::new(*w as u32).unwrap(),
                            NonZeroU32::new(*h as u32).unwrap(),
                        )
                        .unwrap();
                }),
                _ => {}
            }
        }
        true
    }
    fn handle_message(message: Message, window: &mut GlfwBackendWindow) -> bool {
        match message {
            Message::Draw { x, y, buffer } => window.with_surface_mut(|surface| {
                let x = x as usize;
                let y = y as usize;
                let mut wbuffer = surface.buffer_mut().unwrap();
                let ww = wbuffer.width().get() as usize;
                let wh = wbuffer.height().get() as usize;
                let bw = buffer.width();
                let bh = buffer.height();
                let mw = usize::min(ww, bw);
                let mh = usize::min(wh, bh);
                for j in x..x + mh {
                    for i in y..y + mw {
                        let bindex = j * bw + i;
                        let windex = j * ww + i;
                        // Color is in ARGB format
                        let color: [u8; 4] = (&buffer.data[bindex * 4..][..4]).try_into().unwrap();
                        wbuffer[windex] = u32::from_ne_bytes(color);
                    }
                }
                wbuffer.present().unwrap();
            }),
            Message::GetBox { resp } => {
                let (w, h) = window.borrow_window().get_size();
                let _ = resp.send((0, 0, w as u32, h as u32));
            }
            Message::Quit => {
                unsafe { glfw::ffi::glfwPostEmptyEvent() };
                return false;
            }
        }
        true
    }
}

impl Backend for GlfwBackend {
    fn run(&mut self, rx: Receiver<Message>) {
        GlfwBackend::run(self, rx);
    }
}
