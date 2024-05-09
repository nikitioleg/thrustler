use core::ThrustlerBackend;

pub struct Vulkan {}

impl Vulkan {
    pub fn new() -> Self {
        Self {}
    }
}

impl ThrustlerBackend for Vulkan {
    fn init(&mut self) {
        print!("Vulkan back init");
    }
}