use crate::Core;
use erupt::vk1_0 as vk;

/// Calculate image memory requirements for gpu_alloc
pub fn image_memory_req(
    core: &Core,
    image: vk::Image,
    usage: gpu_alloc::UsageFlags,
) -> gpu_alloc::Request {
    request_from_usage_requirements(
        unsafe { core.device.get_image_memory_requirements(image, None) },
        usage,
    )
}

/// Calculate buffer memory requirements for gpu_alloc
pub fn buffer_memory_req(
    core: &Core,
    buffer: vk::Buffer,
    usage: gpu_alloc::UsageFlags,
) -> gpu_alloc::Request {
    request_from_usage_requirements(
        unsafe { core.device.get_buffer_memory_requirements(buffer, None) },
        usage,
    )
}

pub fn request_from_usage_requirements(
    requirements: vk::MemoryRequirements,
    usage: gpu_alloc::UsageFlags,
) -> gpu_alloc::Request {
    gpu_alloc::Request {
        size: requirements.size,
        align_mask: requirements.alignment,
        usage,
        memory_types: requirements.memory_type_bits,
    }
}
