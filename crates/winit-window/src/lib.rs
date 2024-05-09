use std::any::Any;
use std::cell::RefCell;
use std::ops::DerefMut;
use std::sync::Arc;

use error_stack::Result;
use error_stack::ResultExt;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowId};

use core::{Size, ThrustlerWindow, WindowEvent};
use core::errors::ThrustlerWindowError;

pub struct WinitWindow {
    window_state: RefCell<WindowState>,
    size: Size,
}

impl WinitWindow {
    pub fn new(
        size: Size,
        window_supplier: Box<dyn Fn(Arc<dyn OutputWindow>) -> ()>,
    ) -> Result<WinitWindow, ThrustlerWindowError> {
        let event_loop = winit::event_loop::EventLoop::new()
            .attach_printable("Can't create event loop")
            .change_context(ThrustlerWindowError::WindowLoopError)?;

        let window_attrs = WindowAttributes::default()
            .with_title("Thrustler")
            .with_inner_size(PhysicalSize::new(size.width, size.height));

        Ok(Self {
            window_state: RefCell::new(WindowState {
                window: None,
                window_attrs: Some(window_attrs),
                event_loop: Some(event_loop),
                event_dispatcher: None,
                window_supplier,
            }),
            size,
        })
    }

    pub fn run(&self, event_dispatcher: Box<dyn FnMut(WindowEvent)>) -> Result<(), ThrustlerWindowError> {
        let event_loop = {
            self.window_state.borrow_mut().event_loop.take().ok_or(ThrustlerWindowError::WindowLoopError)?
        };

        {
            self.window_state.borrow_mut().event_dispatcher.replace(event_dispatcher);
        }
        event_loop.run_app(self.window_state.borrow_mut().deref_mut())
            .attach_printable("An event loop error has happened")
            .change_context(ThrustlerWindowError::WindowLoopError)
    }
}

impl ThrustlerWindow for WinitWindow {
    fn start(&self, dispatcher: Box<dyn FnMut(WindowEvent) -> ()>) -> Result<(), ThrustlerWindowError> {
        self.run(dispatcher)
    }
}

struct WindowState {
    window: Option<Arc<Window>>,
    window_attrs: Option<WindowAttributes>,
    event_loop: Option<winit::event_loop::EventLoop<()>>,
    event_dispatcher: Option<Box<dyn FnMut(WindowEvent) -> ()>>,
    window_supplier: Box<dyn Fn(Arc<dyn OutputWindow>) -> ()>,
}

impl WindowState {
    fn dispatch_event(&mut self, event: WindowEvent) {
        self.event_dispatcher.as_mut()
            .expect("Event dispatcher doesn't set up")
            (event);
    }
}

impl ApplicationHandler<()> for WindowState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(self.window_attrs.take().unwrap()).unwrap();
        let rc_window = Arc::new(window);
        let trait_object: Arc<dyn OutputWindow> = rc_window.clone() as Arc<dyn OutputWindow>;

        self.window_supplier.as_mut()(trait_object);
        self.dispatch_event(WindowEvent::OnStart);
        self.window = Some(rc_window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                self.dispatch_event(WindowEvent::OnStop);
                event_loop.exit();
            }
            winit::event::WindowEvent::RedrawRequested => {
                self.dispatch_event(WindowEvent::OnDraw);
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window.as_ref().unwrap().clone().request_redraw()
    }
}

pub trait OutputWindow: HasRawWindowHandle + HasRawDisplayHandle + Any + Send + Sync {}

impl OutputWindow for Window {}