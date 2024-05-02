use anyhow::{anyhow, Ok, Result};
use log::{info, warn};
use std::{collections::HashSet, ffi::CStr, os::raw::c_void};
use vulkanalia::{
    loader::{LibloadingLoader, LIBRARY},
    prelude::v1_0::*,
    vk::{
        make_version, ApplicationInfo, Bool32, DebugUtilsMessageSeverityFlagsEXT,
        DebugUtilsMessageTypeFlagsEXT, DebugUtilsMessengerCallbackDataEXT,
        DebugUtilsMessengerCreateInfoEXT, DebugUtilsMessengerEXT, ExtDebugUtilsExtension,
        ExtensionName, InstanceCreateFlags, InstanceCreateInfo, EXT_DEBUG_UTILS_EXTENSION, FALSE,
        KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION, KHR_PORTABILITY_ENUMERATION_EXTENSION,
    },
    window::{self as vk_window, get_required_instance_extensions},
    Instance, Version,
};
use winit::window::Window;

const PORTABILITY_MACOS_VERSION: Version = Version::new(1, 3, 126);

const VALIDATION_ENABLED: bool = cfg!(debug_assertions);
const VALIDATION_LAYER: ExtensionName = ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

extern "system" fn debug_callback(
    severity: DebugUtilsMessageSeverityFlagsEXT,
    type_: DebugUtilsMessageTypeFlagsEXT,
    data: *const DebugUtilsMessengerCallbackDataEXT,
    _: *mut c_void,
) -> Bool32 {
    let data = unsafe { *data };
    let message = unsafe { CStr::from_ptr(data.message) }.to_string_lossy();

    if severity >= DebugUtilsMessageSeverityFlagsEXT::ERROR {
        panic!("({:?}) {}", type_, message);
    } else if severity >= DebugUtilsMessageSeverityFlagsEXT::WARNING {
        warn!("({:?}) {}", type_, message);
    } else if severity >= DebugUtilsMessageSeverityFlagsEXT::INFO {
        dbg!("({:?}) {}", type_, message);
    } else {
        println!("({:?}) {}", type_, message);
    };

    FALSE
}

#[derive(Clone, Debug)]
pub struct App {
    instance: Instance,
    data: AppData,
    entry: Entry,
}

impl App {
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = create_instance(window, &entry, &mut data)?;

        Ok(Self {
            entry,
            data,
            instance,
        })
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    pub unsafe fn destroy(&mut self) {
        self.instance.destroy_instance(None)
    }
}

#[derive(Clone, Debug, Default)]
pub struct AppData {
    messenger: DebugUtilsMessengerEXT,
}

pub unsafe fn create_instance(
    window: &Window,
    entry: &Entry,
    data: &mut AppData,
) -> Result<Instance> {
    let app_info = ApplicationInfo::builder()
        .application_name(b"Vulkan Tutorial\0")
        .application_version(make_version(1, 0, 0))
        .engine_name(b"No Engine\0")
        .engine_version(make_version(1, 0, 0))
        .api_version(make_version(1, 0, 0));

    let mut extensions = get_required_instance_extensions(window)
        .iter()
        .map(|extension| extension.as_ptr())
        .collect::<Vec<_>>();

    let available_layers = entry
        .enumerate_instance_layer_properties()?
        .iter()
        .map(|layer| layer.layer_name)
        .collect::<HashSet<_>>();

    if VALIDATION_ENABLED && !available_layers.contains(&VALIDATION_LAYER) {
        return Err(anyhow!("Validation lauer requested but not suported"));
    }

    if VALIDATION_ENABLED {
        extensions.push(EXT_DEBUG_UTILS_EXTENSION.name.as_ptr());
    }

    let layers = if VALIDATION_ENABLED {
        vec![VALIDATION_LAYER.as_ptr()]
    } else {
        Vec::new()
    };

    let flags = if cfg!(target_os = "macos") && entry.version()? >= PORTABILITY_MACOS_VERSION {
        info!("Enabling extensions for macOS portability.");
        extensions.push(KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_EXTENSION.name.as_ptr());
        extensions.push(KHR_PORTABILITY_ENUMERATION_EXTENSION.name.as_ptr());

        InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
    } else {
        InstanceCreateFlags::empty()
    };

    let info = InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers)
        .enabled_extension_names(&extensions)
        .flags(flags);

    let instance = entry.create_instance(&info, None)?;

    if VALIDATION_ENABLED {
        let debug_info = DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(DebugUtilsMessageTypeFlagsEXT::all())
            .user_callback(Some(debug_callback));

        data.messenger = instance.create_debug_utils_messenger_ext(&debug_info, None)?;
    }

    Ok(instance)
}
