pub use erupt::vk1_0 as vk;
mod core;
pub use crate::core::*;
mod default_engine;
mod windowed;

pub type Memory = gpu_alloc::MemoryBlock<vk::DeviceMemory>;

/// Data associated with CPU-GPU and GPU-GPU synchronization.
/// These synchronization primitives are signalled when a frame finishes.
pub struct FrameSync {
    pub semaphore: vk::Semaphore,
    pub fence: vk::Fence,
    _core: SharedCore,
}

/// A set of meshes which are allocated and deallocated together
pub struct MeshBundle {
    pub vertices: vk::Buffer,
    pub indices: vk::Buffer,
    pub memory: Memory,
    _core: SharedCore,
}

/// Represents a backing pipeline that can render an object
/// with the from which it was created.
pub struct Material {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
    _core: SharedCore,
}

/// Abstraction over a single image; contains view and extent
pub struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub extent: vk::Extent2D,
    _core: SharedCore,
}

/// Abstraction over a single image; contains view and extent 
/// as well as a fence to wait until it is unused. 
pub struct SwapchainImage {
    pub view: vk::ImageView,
    pub extent: vk::Extent2D,
    /// Signalled when this image is unused
    pub fence: vk::Fence,
    _core: SharedCore,
}
