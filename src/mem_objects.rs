use crate::{Core, Memory};
use anyhow::Result;
use drop_bomb::DropBomb;
use erupt::vk1_0 as vk;

pub trait Viewable {
    type View;
}

/// Memory objects that have attached views. Meant for simple cases where there is only one view
/// and one memory object (I.E. Buffer, Image)
pub struct MemObject<T: Viewable> {
    pub instance: T,
    pub memory: Option<Memory>,
    pub view: T::View,
    bomb: DropBomb,
}

impl Viewable for vk::Image {
    type View = vk::ImageView;
}

impl Viewable for vk::Buffer {
    type View = vk::BufferView;
}

impl MemObject<vk::Image> {
    /// Allocate a new image with the given usage. Note that for the view builder, `image` does not
    /// need to be specified as this method will handle adding it.
    pub fn new(
        core: &Core,
        create_info: vk::ImageCreateInfoBuilder<'static>,
        view_create_info: vk::ImageViewCreateInfoBuilder<'static>,
        usage: gpu_alloc::UsageFlags,
    ) -> Result<Self> {
        let instance = unsafe { core.device.create_image(&create_info, None, None) }.result()?;
        let memory = core.allocate(crate::memory::image_memory_req(&core, instance, usage))?;
        unsafe {
            core.device
                .bind_image_memory(instance, *memory.memory(), memory.offset())
                .result()?;
        }
        let view_create_info = view_create_info.image(instance);
        let view =
            unsafe { core.device.create_image_view(&view_create_info, None, None) }.result()?;
        Ok(Self {
            instance,
            memory: Some(memory),
            view,
            bomb: DropBomb::new("Image memory object dropped without calling free()!"),
        })
    }

    pub fn free(&mut self, core: &Core) {
        unsafe {
            core.device.destroy_image(Some(self.instance), None);
            core.device.destroy_image_view(Some(self.view), None);
            core.deallocate(self.memory.take().expect("Double free of image memory"))
                .unwrap();
            self.bomb.defuse();
        }
    }
}

impl MemObject<vk::Buffer> {
    /// Allocate a new buffer with the given usage. Note that for the view builder, `buffer` does not
    /// need to be specified as this method will handle adding it.
    pub fn new(
        core: &Core,
        create_info: vk::BufferCreateInfoBuilder<'static>,
        view_create_info: vk::BufferViewCreateInfoBuilder<'static>,
        usage: gpu_alloc::UsageFlags,
    ) -> Result<Self> {
        let instance = unsafe { core.device.create_buffer(&create_info, None, None) }.result()?;
        let memory = core.allocate(crate::memory::buffer_memory_req(&core, instance, usage))?;
        unsafe {
            core.device
                .bind_buffer_memory(instance, *memory.memory(), memory.offset())
                .result()?;
        }
        let view_create_info = view_create_info.buffer(instance);
        let view =
            unsafe { core.device.create_buffer_view(&view_create_info, None, None) }.result()?;
        Ok(Self {
            instance,
            memory: Some(memory),
            view,
            bomb: DropBomb::new("Buffer memory object dropped without calling free()!"),
        })
    }

    pub fn free(&mut self, core: &Core) {
        unsafe {
            core.device.destroy_buffer(Some(self.instance), None);
            core.device.destroy_buffer_view(Some(self.view), None);
            core.deallocate(self.memory.take().expect("Double free of buffer memory"))
                .unwrap();
            self.bomb.defuse();
        }
    }
}
