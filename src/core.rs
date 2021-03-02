use anyhow::{format_err, Result};
use erupt::{utils::loading::DefaultEntryLoader, vk1_0 as vk, DeviceLoader, InstanceLoader};
use gpu_alloc::{GpuAllocator, Request};
use gpu_alloc_erupt::EruptMemoryDevice as EMD;

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

    pub fn allocate(&self, request: Request) -> Result<Memory> {
        unsafe {
            Ok(self
                .allocator()?
                .alloc(EMD::wrap(&self.device), request)?)
        }
    }

    pub fn deallocate(&self, memory: Memory) -> Result<()> {
        unsafe {
            Ok(self
                .allocator()?
                .dealloc(EMD::wrap(&self.device), memory))
        }
    }
}

/// A simple pointer into the core
pub type SharedCore = Arc<Core>;
