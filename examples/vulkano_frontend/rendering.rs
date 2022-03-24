use vulkano::{
    device::{physical::PhysicalDevice, Device},
    format::Format,
    image::{ImageDimensions::Dim2d, StorageImage},
    sync::{FenceSignalFuture, GpuFuture},
};

use std::sync::{atomic::AtomicBool, Arc, Mutex};

pub struct RenderInfo {
    pub render_width: u32,
    pub render_height: u32,
}

impl RenderInfo {
    pub fn new(render_width: u32, render_height: u32) -> Self {
        RenderInfo {
            render_width,
            render_height,
        }
    }
}

pub struct CpuRendering {
    pub cpu_swapchain: [Arc<StorageImage>; 2],
    pub to_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
    pub from_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
    pub copy_to_first: Arc<AtomicBool>,
}

impl CpuRendering {
    pub fn new(
        physical_device: &PhysicalDevice,
        device: Arc<Device>,
        width: u32,
        height: u32,
    ) -> Self {
        let mut usage = vulkano::image::ImageUsage::none();
        usage.storage = true;
        usage.transfer_source = true;
        usage.transfer_destination = true;

        let cpu_swapchain = [
            StorageImage::with_usage(
                device.clone(),
                Dim2d {
                    width,
                    height,
                    array_layers: 1,
                },
                Format::R32G32B32A32_SFLOAT,
                usage,
                vulkano::image::ImageCreateFlags::none(),
                physical_device.queue_families(),
            )
            .unwrap(),
            StorageImage::with_usage(
                device.clone(),
                Dim2d {
                    width,
                    height,
                    array_layers: 1,
                },
                Format::R32G32B32A32_SFLOAT,
                usage,
                vulkano::image::ImageCreateFlags::none(),
                physical_device.queue_families(),
            )
            .unwrap(),
        ];
        CpuRendering {
            cpu_swapchain,
            to_sc: Arc::new(Mutex::new(None)),
            from_sc: Arc::new(Mutex::new(None)),
            copy_to_first: Arc::new(AtomicBool::new(true)),
        }
    }
}
