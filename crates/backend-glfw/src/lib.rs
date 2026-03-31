use glfw::Context as _;
use pocowl_backend::Backend;

pub struct GlfwBackend {}

impl GlfwBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl Backend for GlfwBackend {
    fn run(&mut self) {
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to init GLFW");
        let (mut window, events) = glfw
            .create_window(800, 600, "GLFW", glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");
        window.set_key_polling(true);
        window.make_current();
        while !window.should_close() {
            window.swap_buffers();
            glfw.poll_events();
            for (_, event) in glfw::flush_messages(&events) {
                dbg!(event);
            }
        }
    }
}
