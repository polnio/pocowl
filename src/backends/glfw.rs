use super::{Backend, Message};
use crate::utils::Geometry;
use crossbeam::channel::Receiver;
use glfw::Context as _;
use ouroboros::self_referencing;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};

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

        let window = Arc::new(Mutex::new(window));

        let (events_tx, events_rx) = crossbeam::channel::unbounded::<Vec<glfw::WindowEvent>>();

        let (stop_tx, stop_rx) = crossbeam::channel::bounded(1);

        std::thread::spawn({
            let window = Arc::clone(&window);
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
                        recv(stop_rx) -> _ => false,
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
        let _ = stop_tx.send(());
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
            Message::WithBuffer { f } => window.with_surface_mut(|surface| {
                let mut buffer = surface.buffer_mut().unwrap();
                let w = buffer.width().get();
                let h = buffer.height().get();
                f(&mut *buffer, w, h);
                buffer.present().unwrap();
            }),
            Message::GetBox { resp } => {
                let (w, h) = window.borrow_window().get_size();
                let _ = resp.send(Geometry {
                    x: 0,
                    y: 0,
                    w: w as u32,
                    h: h as u32,
                });
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
