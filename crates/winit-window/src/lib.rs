use std::cell::RefCell;
use std::thread::sleep;
use std::time::Duration;

use core::{ThrustlerWindow, WindowEvent};

pub struct WinitWindow {
    window_state: RefCell<WindowState>,
}

struct WindowState {
    event_loop: Option<EventLoop>,
}

struct EventLoop();

impl EventLoop {
    pub fn run(self, mut dispatcher: Box<dyn FnMut(WindowEvent)>) {
        let mut number = 0;

        dispatcher.as_mut()(WindowEvent::OnStart);
        loop {
            sleep(Duration::from_secs(1));
            println!("iteration {}", number);
            dispatcher.as_mut()(WindowEvent::OnDraw);
            number = number + 1;
        }
        dispatcher.as_mut()(WindowEvent::OnStop);
    }
}

impl WinitWindow {
    pub fn new() -> Self {
        let window_state = WindowState {
            event_loop: Some(EventLoop()),
        };

        Self {
            window_state: RefCell::new(window_state)
        }
    }
}

impl ThrustlerWindow for WinitWindow {
    fn start(&self, dispatcher: Box<dyn FnMut(WindowEvent) -> ()>) {
        let mut window_state = self.window_state.borrow_mut();
        window_state.event_loop.take().unwrap().run(dispatcher);
    }
}