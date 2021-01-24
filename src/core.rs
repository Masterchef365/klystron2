use erupt::{
    utils::loading::DefaultEntryLoader, vk1_0 as vk, DeviceLoader, EntryLoader, InstanceLoader,
};
use gpu_alloc::GpuAllocator;
use std::sync::{Arc, Mutex};

/// A collection of commonly referenced resources
pub struct Core {
    pub utility_queue: vk::Queue,
    pub graphics_queue: vk::Queue,
    pub allocator: Mutex<GpuAllocator<vk::DeviceMemory>>,
    pub device: DeviceLoader,
    pub instance: InstanceLoader,
    pub _entry: DefaultEntryLoader,
}

/// A simple pointer into the core
pub type SharedCore = Arc<Core>;
