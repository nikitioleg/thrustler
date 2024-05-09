pub trait ThrustlerWindow {
    fn start(&self, dispatcher: Box<dyn FnMut(WindowEvent) -> ()>);
}

pub enum WindowEvent{
    OnStart,
    OnDraw,
    OnStop,
}

pub trait ThrustlerBackend {
    fn init(&mut self);
}
