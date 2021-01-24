pub use erupt::vk1_0 as vk;
mod core;
pub use crate::core::*;
mod default_engine;

pub type Memory = gpu_alloc::MemoryBlock<vk::DeviceMemory>;

/// Data associated with CPU-GPU and GPU-GPU synchronization.
/// These synchronization primitives are signalled when a frame finishes.
pub struct FrameSync {
    pub semaphore: vk::Semaphore,
    pub fence: vk::Fence,
    prelude: SharedCore,
}

/// A set of meshes which are allocated and deallocated together
pub struct MeshBundle {
    pub vertices: vk::Buffer,
    pub indices: vk::Buffer,
    pub memory: Memory,
    prelude: SharedCore,
}

/// Represents a backing pipeline that can render an object
/// with the from which it was created.
pub struct Material {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    prelude: SharedCore,
}

