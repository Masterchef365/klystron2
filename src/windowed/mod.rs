use crate::*;
use anyhow::{format_err, Result};
use erupt::{
    extensions::{
        khr_surface::{self, SurfaceKHR, ColorSpaceKHR},
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
use hardware::SurfaceInfo;
use winit::window::Window;

pub const COLOR_FORMAT: vk::Format = vk::Format::B8G8R8A8_SRGB;
pub const COLOR_SPACE: ColorSpaceKHR = ColorSpaceKHR::SRGB_NONLINEAR_KHR;

/// Add extensions to `setup` needed to accomodate `window`
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

/// Select an appropriate image count
pub fn image_count(surface_caps: khr_surface::SurfaceCapabilitiesKHR) -> u32 {
    let mut image_count = surface_caps.min_image_count + 1;
    if surface_caps.max_image_count > 0 && image_count > surface_caps.max_image_count {
        image_count = surface_caps.max_image_count;
    }
    image_count
}

/// Find appropriate hardware, and create a core
pub fn basics(
    app_info: &ApplicationInfo,
    setup: &mut VulkanSetup,
    window: &Window,
) -> Result<(SurfaceKHR, HardwareSelection, SurfaceInfo, SharedCore)> {
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
    let (hardware, surface_info) = hardware::query(&instance, surface, &setup.device_extensions)?;

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

    Ok((surface, hardware, surface_info, core))
}
