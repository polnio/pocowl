use glfw::Context as _;
use ouroboros::self_referencing;
use pocowl_backend::{Backend, BackendSender, Message};
use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use tokio::sync::mpsc::Receiver;

#[self_referencing]
pub struct GlfwBackendWindow {
    window: glfw::PWindow,
    #[borrows(window)]
    #[covariant]
    surface: softbuffer::Surface<&'this glfw::PWindow, &'this glfw::PWindow>,
}
pub struct GlfwBackend {
    rx: Receiver<Message>,
    glfw: Glfw,
    events: GlfwReceiver<(f64, glfw::WindowEvent)>,
    window: GlfwBackendWindow,
}
impl GlfwBackend {
    fn new(rx: Receiver<Message>) -> Self {
        let glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init GLFW");
        let mut glfw = Glfw(glfw);
        let (mut window, events) = glfw
            .create_window(800, 600, "GLFW", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");
        let events = GlfwReceiver(events);
        window.set_size_polling(true);
        window.make_current();

        let window = GlfwBackendWindow::new(window, |window| {
            let sbctx =
                softbuffer::Context::new(window).expect("Failed to create softbuffer context");
            let surface =
                softbuffer::Surface::new(&sbctx, window).expect("Failed to create surface");
            surface
        });

        Self {
            rx,
            glfw,
            events,
            window,
        }
    }

    async fn run(&mut self) {
        self.window.with_surface_mut(|surface| {
            surface
                .resize(NonZeroU32::new(800).unwrap(), NonZeroU32::new(600).unwrap())
                .unwrap();
        });

        while !self.window.borrow_window().should_close() {
            tokio::select! {
                events = self.glfw.get_events(&self.events) =>  self.handle_events(&events),
                Some(message) = self.rx.recv() => self.handle_message(message)
            }
        }
    }

    fn handle_events(&mut self, events: &[glfw::WindowEvent]) {
        for event in events {
            match event {
                glfw::WindowEvent::Size(w, h) => self.window.with_surface_mut(|surface| {
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
    }
    fn handle_message(&mut self, message: Message) {
        match message {
            Message::Draw { x, y, buffer } => self.window.with_surface_mut(|surface| {
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
                let (w, h) = self.window.borrow_window().get_size();
                let _ = resp.send((0, 0, w as u32, h as u32));
            }
        }
    }
}

impl Backend for GlfwBackend {
    fn new_pair() -> (Self, BackendSender)
    where
        Self: Sized,
    {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        (Self::new(rx), BackendSender::new(tx))
    }

    async fn run(&mut self) {
        GlfwBackend::run(self).await;
    }
}

struct Glfw(glfw::Glfw);
impl Glfw {
    async fn get_events(
        &mut self,
        events_rx: &GlfwReceiver<(f64, glfw::WindowEvent)>,
    ) -> Vec<glfw::WindowEvent> {
        let mut events = Vec::new();
        while events.is_empty() {
            self.0.poll_events();
            events = glfw::flush_messages(events_rx).map(|(_, e)| e).collect();
            tokio::task::yield_now().await;
        }
        events
    }
}
impl Deref for Glfw {
    type Target = glfw::Glfw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for Glfw {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
unsafe impl Send for Glfw {}
struct GlfwReceiver<T>(glfw::GlfwReceiver<T>);
impl<T> Deref for GlfwReceiver<T> {
    type Target = glfw::GlfwReceiver<T>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for GlfwReceiver<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
unsafe impl<T> Send for GlfwReceiver<T> {}
unsafe impl<T> Sync for GlfwReceiver<T> {}
