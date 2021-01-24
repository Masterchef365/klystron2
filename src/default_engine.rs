use crate::core::SharedCore;
use crate::*;
use slotmap::{SlotMap, DefaultKey};

/// Number of frames in-flight. >1 means the GPU and CPU work in parallel
const N_FRAMES: usize = 2;

// Thinking about doing another layer of encapsulation... Must resist
// My thought was that the concept of a mesh bundle is higher-level than the rest of the renderer
// and would be subject to change much more often

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
