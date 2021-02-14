use anyhow::{format_err, Result};
use erupt::{utils::loading::DefaultEntryLoader, vk1_0 as vk, DeviceLoader, InstanceLoader};
use gpu_alloc::GpuAllocator;
use gpu_alloc_erupt::EruptMemoryDevice;
use std::sync::MutexGuard;
use std::sync::{Arc, Mutex};

pub type Memory = gpu_alloc::MemoryBlock<vk::DeviceMemory>;

/// A collection of commonly referenced resources
pub struct Core {
    pub utility_queue: vk::Queue,
    pub graphics_queue: vk::Queue,
    pub allocator: Mutex<GpuAllocator<vk::DeviceMemory>>,
    pub device: DeviceLoader,
    pub instance: InstanceLoader,
    pub _entry: DefaultEntryLoader,
}

impl Core {
    pub fn allocator(&self) -> Result<MutexGuard<GpuAllocator<vk::DeviceMemory>>> {
        self.allocator
            .lock()
            .map_err(|_| format_err!("GpuAllocator mutex poisoned"))
    }

    pub fn allocate(&self, request: gpu_alloc::Request) -> Result<Memory> {
        unsafe {
            Ok(self
                .allocator()?
                .alloc(EruptMemoryDevice::wrap(&self.device), request)?)
        }
    }
}

/// A simple pointer into the core
pub type SharedCore = Arc<Core>;
