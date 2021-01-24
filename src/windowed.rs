use crate::*;
use anyhow::{format_err, Result};
use erupt::{
    extensions::{khr_surface, khr_swapchain},
    InstanceLoader,
};
use std::{ffi::CStr, os::raw::c_char};

const COLOR_FORMAT: vk::Format = vk::Format::B8G8R8A8_SRGB;

pub struct Windowed {
    pub swapchain: Option<khr_swapchain::SwapchainKHR>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub surface: khr_surface::SurfaceKHR,
    pub hardware: HardwareSelection,
    _prelude: SharedCore,
}

/// Hardware selection for Winit backend
#[derive(Debug)]
pub struct HardwareSelection {
    pub physical_device: vk::PhysicalDevice,
    pub physical_device_properties: vk::PhysicalDeviceProperties,
    pub graphics_queue_family: u32,
    pub utility_queue_family: u32,
    pub format: khr_surface::SurfaceFormatKHR,
    pub present_mode: khr_surface::PresentModeKHR,
}

/// Finds a GRAPHICS queue that the device supports
pub fn find_surface_queue_family(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    surface: khr_surface::SurfaceKHR,
) -> Result<u32> {
    let qf_properties = unsafe {
        instance
            .get_physical_device_queue_family_properties(physical_device, None)
            .into_iter()
    };
    for (i, properties) in qf_properties.enumerate() {
        let has_graphics = properties.queue_flags.contains(vk::QueueFlags::GRAPHICS);
        let supports_surface = unsafe {
            instance
                .get_physical_device_surface_support_khr(physical_device, i as u32, surface, None)
                .result()?
        };

        if has_graphics && supports_surface {
            return Ok(i as u32);
        }
    }
    Err(format_err!("No suitable graphics queue family found"))
}

/// Finds a COMPUTE queue (also for transfer)
pub fn find_utility_queue_family(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
) -> Result<u32> {
    let qf_properties = unsafe {
        instance
            .get_physical_device_queue_family_properties(physical_device, None)
            .into_iter()
    };
    for (i, properties) in qf_properties.enumerate() {
        let has_compute = properties.queue_flags.contains(vk::QueueFlags::GRAPHICS);
        let has_transfer = properties.queue_flags.contains(vk::QueueFlags::TRANSFER);

        if has_compute && has_transfer {
            return Ok(i as u32);
        }
    }
    Err(format_err!("No suitable graphics queue family found"))
}

/// Find a surface format compatible with COLOR_FORMAT (and SRGB_NONLINEAR_KHR if available)
pub fn select_surface_format(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    surface: khr_surface::SurfaceKHR,
) -> Result<khr_surface::SurfaceFormatKHR> {
    let formats = unsafe {
        instance
            .get_physical_device_surface_formats_khr(physical_device, surface, None)
            .result()?
    };
    match formats
        .iter()
        .find(|surface_format| {
            surface_format.format == vk::Format::B8G8R8A8_SRGB
                && surface_format.color_space == khr_surface::ColorSpaceKHR::SRGB_NONLINEAR_KHR
        })
        .or_else(|| formats.get(0))
    {
        Some(surface_format) => Ok(surface_format.clone()),
        None => return Err(format_err!("No suitable surface format found.")),
    }
}

pub fn select_present_mode(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    surface: khr_surface::SurfaceKHR,
) -> Result<khr_surface::PresentModeKHR> {
    unsafe {
        Ok(instance
            .get_physical_device_surface_present_modes_khr(physical_device, surface, None)
            .result()?
            .into_iter()
            .find(|present_mode| present_mode == &khr_surface::PresentModeKHR::MAILBOX_KHR)
            .unwrap_or(khr_surface::PresentModeKHR::FIFO_KHR))
    }
}

pub fn check_supported_extensions(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    device_extensions: &[*const c_char],
) -> Result<Vec<vk::ExtensionProperties>> {
    let supported_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device, None, None)
            .result()?
    };
    let all_supported = device_extensions.iter().all(|device_extension| {
        let device_extension = unsafe { CStr::from_ptr(*device_extension) };

        supported_extensions.iter().any(|properties| {
            let extension_name = unsafe { CStr::from_ptr(properties.extension_name.as_ptr()) };
            extension_name == device_extension
        })
    });
    match all_supported {
        false => Err(format_err!("Extension ")),
        true => Ok(supported_extensions),
    }
}

pub fn select_hardware_physical_device(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    surface: khr_surface::SurfaceKHR,
    device_extensions: &[*const c_char],
) -> Result<HardwareSelection> {
    Ok(HardwareSelection {
        physical_device,
        graphics_queue_family: find_surface_queue_family(instance, physical_device, surface)?,
        utility_queue_family: find_utility_queue_family(instance, physical_device)?,
        format: select_surface_format(instance, physical_device, surface)?,
        present_mode: select_present_mode(instance, physical_device, surface)?,
        physical_device_properties: unsafe {
            instance.get_physical_device_properties(physical_device, None)
        },
    })
}

pub fn score_hardware_config(hardware: &HardwareSelection) -> u32 {
    match hardware.physical_device_properties.device_type {
        vk::PhysicalDeviceType::DISCRETE_GPU => 2,
        vk::PhysicalDeviceType::INTEGRATED_GPU => 1,
        _ => 0,
    }
}

/// Query for hardware with the right properties for windowed mode
pub fn query(
    instance: &InstanceLoader,
    surface: khr_surface::SurfaceKHR,
    device_extensions: &[*const c_char],
) -> Result<HardwareSelection> {
    unsafe { instance.enumerate_physical_devices(None) }
        .unwrap()
        .into_iter()
        .filter_map(|physical_device| {
            select_hardware_physical_device(instance, physical_device, surface, device_extensions)
                .ok()
        })
        .max_by_key(score_hardware_config)
        .ok_or_else(|| anyhow::format_err!("No suitable hardware found for this configuration"))
}
