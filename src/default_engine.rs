use crate::core::SharedCore;
use crate::*;
use slotmap::{DefaultKey, SlotMap};
use erupt::cstr;

/// Number of frames in-flight. >1 means the GPU and CPU work in parallel
const N_FRAMES: usize = 2;

pub struct Engine {
    pub swapchain_images: Vec<SwapchainImage>,
    pub depth_image: Image,
    pub command_buffers: [vk::CommandBuffer; N_FRAMES],
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub descriptor_pool: vk::DescriptorPool,
    pub command_pool: vk::CommandPool,
    pub render_pass: vk::RenderPass,
    pub materials: SlotMap<DefaultKey, Material>,
    pub meshes: SlotMap<DefaultKey, MeshBundle>,
    pub frame_sync: [FrameSync; N_FRAMES],
    /// The index of the frame that is currently writeable
    /// (or equivalently: not in-use by the GPU).
    pub frame_idx: usize,
    pub _core: SharedCore,
}

pub fn vk_setup(validation: bool) -> VulkanSetup {
    const LAYER_KHRONOS_VALIDATION: *const i8 = cstr!("VK_LAYER_KHRONOS_validation");
    use erupt::extensions::ext_debug_utils::EXT_DEBUG_UTILS_EXTENSION_NAME;
    let api_version = vk::make_version(1, 0, 0);
    if validation {
        VulkanSetup {
            instance_layers: vec![],
            instance_extensions: vec![],
            device_layers: vec![],
            device_extensions: vec![],
            api_version,
        }
    } else {
        VulkanSetup {
            instance_layers: vec![LAYER_KHRONOS_VALIDATION],
            instance_extensions: vec![EXT_DEBUG_UTILS_EXTENSION_NAME],
            device_layers: vec![LAYER_KHRONOS_VALIDATION],
            device_extensions: vec![],
            api_version,
        }
    }
}
