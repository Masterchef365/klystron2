use erupt::extensions::{khr_surface, khr_swapchain};
pub use erupt::vk1_0 as vk;
mod core;
pub use crate::core::*;
pub mod default_engine;
pub mod windowed;
pub mod mem_objects;
pub mod memory;

pub const ENGINE_NAME: &str = "Klystron II";

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

/// Set of selected hardware properties
#[derive(Debug)]
pub struct HardwareSelection {
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub graphics_queue_family: u32,
    pub utility_queue_family: u32,
}

/// Set of Vulkan layers, extensions, and version
pub struct VulkanSetup {
    pub instance_layers: Vec<*const i8>,
    pub instance_extensions: Vec<*const i8>,
    pub device_layers: Vec<*const i8>,
    pub device_extensions: Vec<*const i8>,
    pub api_version: u32,
}

impl VulkanSetup {
    pub fn validation(api_version: u32) -> Self {
        const LAYER_KHRONOS_VALIDATION: *const i8 = erupt::cstr!("VK_LAYER_KHRONOS_validation");
        use erupt::extensions::ext_debug_utils::EXT_DEBUG_UTILS_EXTENSION_NAME;
        VulkanSetup {
            instance_layers: vec![LAYER_KHRONOS_VALIDATION],
            instance_extensions: vec![EXT_DEBUG_UTILS_EXTENSION_NAME],
            device_layers: vec![LAYER_KHRONOS_VALIDATION],
            device_extensions: vec![],
            api_version,
        }
    }
}

impl Default for VulkanSetup {
    /// Defaults to Vulkan 1.0.0
    fn default() -> Self {
        Self {
            instance_layers: Vec::new(),
            instance_extensions: Vec::new(),
            device_layers: Vec::new(),
            device_extensions: Vec::new(),
            api_version: vk::make_version(1, 0, 0),
        }
    }
}

/// Application info to be passed to instance creation
pub struct ApplicationInfo {
    pub name: String,
    pub version: u32,
}

macro_rules! cargo_vk_version {
    () => {
        vk::make_version(
            env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(1),
            env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0),
            env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0),
        )
    };
}

/// Return the vulkan-ready version of this engine
pub fn engine_version() -> u32 {
    cargo_vk_version!()
}
