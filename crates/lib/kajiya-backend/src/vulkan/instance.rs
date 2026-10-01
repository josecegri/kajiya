use anyhow::Result;
use ash::{ext, vk};
#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};
use std::{
    borrow::Cow,
    ffi::{self, CStr, CString},
    sync::Arc,
};

#[derive(Default)]
pub struct DeviceBuilder {
    pub required_extensions: Vec<&'static CStr>,
    pub graphics_debugging: bool,
}

impl DeviceBuilder {
    pub fn build(self) -> Result<Arc<Instance>> {
        Ok(Arc::new(Instance::create(self)?))
    }

    pub fn required_extensions(mut self, required_extensions: Vec<&'static CStr>) -> Self {
        self.required_extensions = required_extensions;
        self
    }

    pub fn graphics_debugging(mut self, graphics_debugging: bool) -> Self {
        self.graphics_debugging = graphics_debugging;
        self
    }
}

pub struct Instance {
    pub(crate) entry: ash::Entry,
    pub raw: ash::Instance,
    #[allow(dead_code)]
    pub(crate) debug_callback: Option<vk::DebugUtilsMessengerEXT>,
    #[allow(dead_code)]
    pub(crate) debug_utils: Option<ext::debug_utils::Instance>,
}

impl Instance {
    pub fn builder() -> DeviceBuilder {
        DeviceBuilder::default()
    }

    fn extension_names(builder: &DeviceBuilder) -> Vec<*const i8> {
        let mut names = vec![];

        if builder.graphics_debugging {
            names.push(ext::debug_utils::NAME.as_ptr());
        }

        names
    }

    fn layer_names(builder: &DeviceBuilder) -> Vec<CString> {
        let mut layer_names = Vec::new();
        if builder.graphics_debugging {
            layer_names.push(CString::new("VK_LAYER_KHRONOS_validation").unwrap());
        }
        layer_names
    }

    fn create(builder: DeviceBuilder) -> Result<Self> {
        let entry = ash::Entry::linked();
        let instance_extensions = builder
            .required_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .chain(Self::extension_names(&builder))
            .collect::<Vec<_>>();

        let layer_names = Self::layer_names(&builder);
        let layer_names: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let app_desc = vk::ApplicationInfo::default().api_version(vk::make_api_version(0, 1, 4, 0));

        let instance_desc = vk::InstanceCreateInfo::default()
            .application_info(&app_desc)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&instance_extensions);

        let instance = unsafe { entry.create_instance(&instance_desc, None)? };
        info!("Created a Vulkan instance");

        let (debug_utils, debug_callback) = if builder.graphics_debugging {
            let debug_utils = ash::ext::debug_utils::Instance::new(&entry, &instance);

            let debug_info = ash::vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
                )
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_callback = unsafe {
                debug_utils
                    .create_debug_utils_messenger(&debug_info, None)
                    .unwrap()
            };

            (Some(debug_utils), Some(debug_callback))
        } else {
            (None, None)
        };

        Ok(Self {
            entry,
            raw: instance,
            debug_callback,
            debug_utils,
        })
    }
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT<'_>,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = unsafe { *p_callback_data };

    let message_id_number = callback_data.message_id_number;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::Borrowed("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy() }
    };

    let message = if callback_data.p_message.is_null() {
        Cow::Borrowed("")
    } else {
        unsafe { ffi::CStr::from_ptr(callback_data.p_message).to_string_lossy() }
    };

    if !message_id_name.contains("Loader Message") {
        println!(
            "[{message_severity:?} ({message_type:?})] [{message_id_name} ({message_id_number})] : {message}\n",
        );
    }

    vk::FALSE
}
