use super::{COLOR_FORMAT, COLOR_SPACE};
use crate::*;
use anyhow::{format_err, Result};
use erupt::{
    extensions::khr_surface,
    InstanceLoader,
};
use std::{ffi::CStr, os::raw::c_char};

/// Surface info for a given device
pub struct SurfaceInfo {
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

/// Finds a COMPUTE and TRANSFER queue
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
        let has_compute = properties.queue_flags.contains(vk::QueueFlags::COMPUTE);
        let has_transfer = properties.queue_flags.contains(vk::QueueFlags::TRANSFER);

        if has_compute && has_transfer {
            return Ok(i as u32);
        }
    }
    Err(format_err!("No suitable utility queue family found"))
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
        .into_iter()
        .find(|&surface_format| {
            surface_format.format == COLOR_FORMAT
                && surface_format.color_space == COLOR_SPACE
        })
    {
        Some(surface_format) => Ok(surface_format),
        None => return Err(format_err!("No suitable surface format found.")),
    }
}

/// Select a present mode for this surface
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

/// Check that the given physical_device supports all of requested_extensions
pub fn check_supported_extensions(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    requested_extensions: &[*const c_char],
) -> Result<Vec<vk::ExtensionProperties>> {
    let supported_extensions = unsafe {
        instance
            .enumerate_device_extension_properties(physical_device, None, None)
            .result()?
    };
    let mut unsupported_extensions = Vec::new();
    for request in requested_extensions {
        let request = unsafe { CStr::from_ptr(*request) };
        let is_supported = supported_extensions.iter().any(|properties| {
            let extension_name = unsafe { CStr::from_ptr(properties.extension_name.as_ptr()) };
            extension_name == request
        });
        if !is_supported {
            unsupported_extensions.push(request);
        }
    }
    if unsupported_extensions.is_empty() {
        Ok(supported_extensions)
    } else {
        Err(format_err!(
            "Unsupported extensions requested: {:?}",
            unsupported_extensions
        ))
    }
}

/// Select hardware given a physical device
pub fn select_hardware_physical_device(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    surface: khr_surface::SurfaceKHR,
) -> Result<HardwareSelection> {
    Ok(HardwareSelection {
        physical_device,
        graphics_queue_family: find_surface_queue_family(instance, physical_device, surface)?,
        utility_queue_family: find_utility_queue_family(instance, physical_device)?,
        physical_device_properties: unsafe {
            instance.get_physical_device_properties(physical_device, None)
        },
    })
}

/// Find a valid SurfaceInfo for this surface
pub fn select_surface_info(
    instance: &InstanceLoader,
    physical_device: vk::PhysicalDevice,
    surface: khr_surface::SurfaceKHR,
) -> Result<SurfaceInfo> {
    Ok(SurfaceInfo {
        format: select_surface_format(instance, physical_device, surface)?,
        present_mode: select_present_mode(instance, physical_device, surface)?,
    })
}

/// Score hardware based on device type
pub fn score_hardware_config(hardware: &HardwareSelection) -> i32 {
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
) -> Result<(HardwareSelection, SurfaceInfo)> {
    unsafe { instance.enumerate_physical_devices(None) }
        .result()?
        .into_iter()
        .map(|physical_device| {
            let hardware = select_hardware_physical_device(instance, physical_device, surface)?;
            let surface = select_surface_info(instance, physical_device, surface)?;
            let _ = check_supported_extensions(instance, physical_device, device_extensions)?;
            Ok((hardware, surface))
        })
        .max_by_key(|res| match res {
            Ok((hardware, _)) => score_hardware_config(&hardware),
            Err(_) => std::i32::MIN,
        })
        .unwrap_or_else(|| Err(format_err!("No physical devices found")))
}
