use anyhow::Result;
use ash::{khr, vk};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::sync::Arc;

pub struct Surface {
    pub(crate) raw: vk::SurfaceKHR,
    pub(crate) fns: khr::surface::Instance,
}

impl Surface {
    pub fn create<W>(instance: &super::instance::Instance, window: &W) -> Result<Arc<Self>>
    where
        W: HasDisplayHandle + HasWindowHandle,
    {
        let surface = unsafe {
            ash_window::create_surface(
                &instance.entry,
                &instance.raw,
                window.display_handle().unwrap().as_raw(),
                window.window_handle().unwrap().as_raw(),
                None,
            )?
        };
        let surface_loader = khr::surface::Instance::new(&instance.entry, &instance.raw);

        Ok(Arc::new(Self {
            raw: surface,
            fns: surface_loader,
        }))
    }
}
