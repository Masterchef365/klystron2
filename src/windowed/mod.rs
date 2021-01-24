use crate::*;
use anyhow::{format_err, Result};
use erupt::{
    extensions::{
        khr_surface::{self, SurfaceKHR},
        khr_swapchain,
    },
    utils::surface,
    DeviceLoader, EntryLoader, InstanceLoader,
};
use std::sync::Mutex;
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};
pub mod hardware;
use winit::window::Window;

const COLOR_FORMAT: vk::Format = vk::Format::B8G8R8A8_SRGB;

pub struct Windowed {
    pub swapchain: Option<khr_swapchain::SwapchainKHR>,
    /// Signalled when the swapchain is finished using a given image
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub surface: khr_surface::SurfaceKHR,
    core: SharedCore,
}

pub fn extensions(setup: &mut VulkanSetup, window: &Window) -> Result<()> {
    setup.instance_extensions.extend(
        surface::enumerate_required_extensions(window)
            .result()?
            .into_iter(),
    );
    setup
        .device_extensions
        .push(khr_swapchain::KHR_SWAPCHAIN_EXTENSION_NAME);
    Ok(())
}

/// Find appropriate hardware, and create a core
pub fn basics(
    app_info: &ApplicationInfo,
    setup: &mut VulkanSetup,
    window: &Window,
) -> Result<(SurfaceKHR, HardwareSelection, SharedCore)> {
    // Entry
    let entry = EntryLoader::new()?;

    // Instance
    let application_name = CString::new(app_info.name.clone())?;
    let engine_name = CString::new(crate::ENGINE_NAME)?;
    let app_info = vk::ApplicationInfoBuilder::new()
        .application_name(&application_name)
        .application_version(vk::make_version(1, 0, 0))
        .engine_name(&engine_name)
        .engine_version(crate::engine_version())
        .api_version(vk::make_version(1, 0, 0));

    // Gather needed extensions
    extensions(setup, window)?;

    // Instance creation
    let create_info = vk::InstanceCreateInfoBuilder::new()
        .application_info(&app_info)
        .enabled_extension_names(&setup.instance_extensions)
        .enabled_layer_names(&setup.instance_layers);

    let mut instance = InstanceLoader::new(&entry, &create_info, None)?;

    // Surface
    let surface = unsafe { surface::create_surface(&mut instance, window, None) }.result()?;

    // Hardware selection
    let hardware = hardware::query(&instance, surface, &setup.device_extensions)?;

    // Create logical device
    let create_info = [vk::DeviceQueueCreateInfoBuilder::new()
        .queue_family_index(hardware.graphics_queue_family)
        .queue_priorities(&[1.0])];

    let physical_device_features = vk::PhysicalDeviceFeaturesBuilder::new();
    let create_info = vk::DeviceCreateInfoBuilder::new()
        .queue_create_infos(&create_info)
        .enabled_features(&physical_device_features)
        .enabled_extension_names(&setup.device_extensions)
        .enabled_layer_names(&setup.device_layers);

    let device = DeviceLoader::new(&instance, hardware.physical_device, &create_info, None)?;

    // Create queues
    let graphics_queue =
        unsafe { device.get_device_queue(hardware.graphics_queue_family, 0, None) };
    let utility_queue = unsafe { device.get_device_queue(hardware.utility_queue_family, 0, None) };

    // Create allocator
    let device_props =
        unsafe { gpu_alloc_erupt::device_properties(&instance, hardware.physical_device)? };
    let allocator = Mutex::new(gpu_alloc::GpuAllocator::new(
        gpu_alloc::Config::i_am_prototyping(),
        device_props,
    ));

    // Create Core
    let core = SharedCore::new(Core {
        utility_queue,
        graphics_queue,
        device,
        instance,
        allocator,
        _entry: entry,
    });

    Ok((surface, hardware, core))
}

/*
let image_available_semaphores = (0..crate::core::FRAMES_IN_FLIGHT)
.map(|_| {
let create_info = vk::SemaphoreCreateInfoBuilder::new();
unsafe {
prelude
.device
.create_semaphore(&create_info, None, None)
.result()
}
})
.collect::<Result<Vec<_>, _>>()?;

let self = Self {
image_available_semaphores
_core: core,
};

let core = Core::new(prelude.clone(), meta, false)?;

Ok(Self {
swapchain: None,
image_available_semaphores,
hardware,
surface,
prelude,
core,
})
*/

/*
pub fn next_frame(&mut self, packet: &FramePacket, camera: &dyn camera::Camera) -> Result<()> {
    if self.swapchain.is_none() {
        self.create_swapchain()?;
    }
    let swapchain = self.swapchain.unwrap();

    let (frame_idx, frame) = self.core.frame_sync.next_frame()?;

    let image_available = self.image_available_semaphores[frame_idx];
    let image_index = unsafe {
        self.prelude.device.acquire_next_image_khr(
            swapchain,
            u64::MAX,
            Some(image_available),
            None,
            None,
        )
    };

    // Early return and invalidate swapchain
    let image_index = if image_index.raw == vk::Result::ERROR_OUT_OF_DATE_KHR {
        self.free_swapchain()?;
        return Ok(());
    } else {
        image_index.unwrap()
    };

    //let image: crate::swapchain_images::SwapChainImage = todo!();
    let image = {
        self.core
            .swapchain_images
            .as_mut()
            .unwrap()
            .next_image(image_index, &frame)?
    };

    // Write command buffers
    let command_buffer = self.core.write_command_buffers(frame_idx, packet, &image)?;

    // Upload camera matrix and time
    let mut data = [0.0; 32];
    data.iter_mut()
        .zip(
            camera
                .matrix(image.extent.width, image.extent.height)
                .as_slice()
                .iter(),
        )
        .for_each(|(o, i)| *o = *i);
    self.core.update_camera_data(frame_idx, &data)?;

    // Submit to the queue
    let command_buffers = [command_buffer];
    let wait_semaphores = [image_available];
    let signal_semaphores = [frame.render_finished];
    let submit_info = vk::SubmitInfoBuilder::new()
        .wait_semaphores(&wait_semaphores)
        .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
        .command_buffers(&command_buffers)
        .signal_semaphores(&signal_semaphores);
    unsafe {
        self.prelude
            .device
            .reset_fences(&[frame.in_flight_fence])
            .result()?; // TODO: Move this into the swapchain next_image
        self.prelude
            .device
            .queue_submit(
                self.prelude.queue,
                &[submit_info],
                Some(frame.in_flight_fence),
            )
            .result()?;
    }

    // Present to swapchain
    let swapchains = [swapchain];
    let image_indices = [image_index];
    let present_info = khr_swapchain::PresentInfoKHRBuilder::new()
        .wait_semaphores(&signal_semaphores)
        .swapchains(&swapchains)
        .image_indices(&image_indices);

    let queue_result = unsafe {
        self.prelude
            .device
            .queue_present_khr(self.prelude.queue, &present_info)
    };

    if queue_result.raw == vk::Result::ERROR_OUT_OF_DATE_KHR {
        self.free_swapchain()?;
        return Ok(());
    } else {
        queue_result.result()?;
    };

    Ok(())
}

fn free_swapchain(&mut self) -> Result<()> {
    drop(self.core.swapchain_images.take());

    unsafe {
        self.prelude
            .device
            .destroy_swapchain_khr(self.swapchain.take(), None);
    }

    Ok(())
}

fn create_swapchain(&mut self) -> Result<()> {
    let surface_caps = unsafe {
        self.prelude
            .instance
            .get_physical_device_surface_capabilities_khr(
                self.hardware.physical_device,
                self.surface,
                None,
            )
    }
    .result()?;

    let mut image_count = surface_caps.min_image_count + 1;
    if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
        image_count = surface_caps.max_image_count;
    }

    // Build the actual swapchain
    let create_info = khr_swapchain::SwapchainCreateInfoKHRBuilder::new()
        .surface(self.surface)
        .min_image_count(image_count)
        .image_format(crate::core::COLOR_FORMAT)
        .image_color_space(self.hardware.format.color_space)
        .image_extent(surface_caps.current_extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(surface_caps.current_transform)
        .composite_alpha(khr_surface::CompositeAlphaFlagBitsKHR::OPAQUE_KHR)
        .present_mode(self.hardware.present_mode)
        .clipped(true)
        .old_swapchain(khr_swapchain::SwapchainKHR::null());

    let swapchain = unsafe {
        self.prelude
            .device
            .create_swapchain_khr(&create_info, None, None)
    }
    .result()?;
    let swapchain_images = unsafe {
        self.prelude
            .device
            .get_swapchain_images_khr(swapchain, None)
    }
    .result()?;

    self.swapchain = Some(swapchain);

    // TODO: Coagulate these two into one object?
    self.swapchain = Some(swapchain);

    self.core.swapchain_images = Some(SwapchainImages::new(
        self.prelude.clone(),
        surface_caps.current_extent,
        self.core.render_pass,
        swapchain_images,
        false,
    )?);

    Ok(())
}
*/

/*
impl Drop for WinitBackend {
    fn drop(&mut self) {
        unsafe {
            for semaphore in self.image_available_semaphores.drain(..) {
                self.prelude.device.destroy_semaphore(Some(semaphore), None);
            }
            self.free_swapchain().unwrap();
            self.prelude
                .instance
                .destroy_surface_khr(Some(self.surface), None);
        }
    }
}
*/
