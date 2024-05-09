use core::{ThrustlerBackend, ThrustlerWindow, WindowEvent};
use vulkan::Vulkan;
use winit_window::WinitWindow;

pub struct Engine {
    window: Box<dyn ThrustlerWindow>,
    backend: Box<dyn ThrustlerBackend>,
}

impl Engine {
    pub fn new() -> Engine {
        let window = WinitWindow::new();
        let vulkan = Vulkan::new();

        println!("Create engine");

        Self {
            window: Box::new(window),
            backend: Box::new(vulkan),
        }
    }

    pub fn start(mut self) {
        //  let mut rc_self = Rc::new(RefCell::new(self));

        let window = self.window;
        let mut backend = self.backend;


        window.start(Box::new(move |event| {

            let mut_back = backend.as_mut();

            match event {
                WindowEvent::OnStart => on_start(mut_back),
                WindowEvent::OnDraw => on_draw(mut_back),
                WindowEvent::OnStop => on_stop(mut_back),
            }
        }));
    }
}

fn on_start(backend: &mut dyn ThrustlerBackend) {
    println!("on_start")
}

fn on_draw(backend: &mut dyn ThrustlerBackend) {
    println!("on_draw")
}

fn on_stop(backend: &mut dyn ThrustlerBackend) {
    println!("on_stop")
}