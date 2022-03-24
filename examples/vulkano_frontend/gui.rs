use crate::rendering::*;
use crate::HEIGHT;
use crate::WIDTH;

use vulkano::pipeline::ComputePipeline;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet},
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceExtensions, Features, Queue,
    },
    format::Format,
    image::{view::ImageView, ImageDimensions::Dim2d, ImageUsage, StorageImage, SwapchainImage},
    instance::Instance,
    pipeline::{Pipeline, PipelineBindPoint},
    sampler::Filter,
    swapchain::{self, AcquireError, Surface, Swapchain, SwapchainCreationError},
    sync::{self, FlushError, GpuFuture},
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use vulkano_win::VkSurfaceBuild;

use std::{sync::Arc, time::SystemTime};

#[derive(Debug)]
pub enum RenderEvent {
    SampleCompleted,
}

pub struct State {
    pub presentation_finished: Option<Box<dyn GpuFuture + 'static>>,
    pub start: SystemTime,
}

impl State {
    pub fn new() -> Self {
        State {
            presentation_finished: None,
            start: SystemTime::now(),
        }
    }
}

pub struct GUI<'a> {
    pub event_loop: Option<EventLoop<RenderEvent>>,
    surface: Arc<Surface<Window>>,
    pub physical_device: PhysicalDevice<'a>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    compute_pipeline: Arc<ComputePipeline>,
    pub cpu_rendering: CpuRendering,
    render_info: RenderInfo,
    combined_buffer: Arc<StorageImage>,
    pub state: State,
}

impl<'a> GUI<'a> {
    pub fn new(instance: &'a Arc<Instance>) -> Self {
        let event_loop: EventLoop<RenderEvent> = EventLoop::with_user_event();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        let event_loop = Some(event_loop);

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };

        let (physical_device, queue_family) = PhysicalDevice::enumerate(instance)
            .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .unwrap();

        println!(
            "Using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type,
        );

        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &physical_device
                .required_extensions()
                .union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();

        let queue = queues.next().unwrap();

        let caps = surface.capabilities(physical_device).unwrap();

        let (swapchain, images) = {
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();

            let mut format = None;

            for f in caps.supported_formats {
                if f.0 == Format::B8G8R8A8_UNORM {
                    format = Some(f.0);
                }
            }

            let format = format.expect("B8G8R8A8_UNORM not suppported!");

            let dimentions: [u32; 2] = surface.window().inner_size().into();

            let mut usage = ImageUsage::none();
            usage.transfer_destination = true;

            Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count)
                .format(format)
                .dimensions(dimentions)
                .usage(usage)
                .sharing_mode(&queue)
                .composite_alpha(composite_alpha)
                .present_mode(swapchain::PresentMode::Fifo)
                .build()
                .unwrap()
        };

        let render_info = RenderInfo::new(WIDTH, HEIGHT);

        let mut usage = vulkano::image::ImageUsage::none();
        usage.storage = true;
        usage.transfer_source = true;

        let combined_buffer = StorageImage::with_usage(
            device.clone(),
            Dim2d {
                width: WIDTH,
                height: HEIGHT,
                array_layers: 1,
            },
            Format::R8G8B8A8_UNORM,
            usage,
            vulkano::image::ImageCreateFlags::none(),
            physical_device.queue_families(),
        )
        .unwrap();

        let cpu_rendering = CpuRendering::new(&physical_device, device.clone(), WIDTH, HEIGHT);

        mod cs {
            vulkano_shaders::shader! {
                                                                                                                                                                                                                                                ty: "compute",
                                                                                                                                                                                                                                                src:
"#version 460

layout(local_size_x = 32, local_size_y = 32) in;

layout(set = 0, binding = 0, rgba32f) uniform readonly image2D cpu_input;

layout(set = 0, binding = 1, rgba8) uniform writeonly image2D image_output;

void main() {
        vec4 data = sqrt(imageLoad(cpu_input, ivec2(gl_GlobalInvocationID.xy)));
        imageStore(image_output, ivec2(gl_GlobalInvocationID.xy), data);
}"}
        }

        let shader = cs::load(device.clone()).expect("failed to create shader module");

        let compute_pipeline = ComputePipeline::new(
            device.clone(),
            shader.entry_point("main").unwrap(),
            &(),
            None,
            |_| {},
        )
        .unwrap();

        GUI {
            event_loop,
            surface,
            physical_device,
            device,
            queue,
            swapchain,
            images,
            render_info,
            compute_pipeline,
            cpu_rendering,
            combined_buffer,
            state: State::new(),
        }
    }

    pub fn run(mut self) {
        use winit::platform::run_return::EventLoopExtRunReturn;
        let mut event_loop = self.event_loop.take().unwrap();
        event_loop.run_return(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;
            match event {
                Event::DeviceEvent {
                    event: winit::event::DeviceEvent::Key(key),
                    ..
                } => match key.virtual_keycode {
                    Some(code) => match code {
                        winit::event::VirtualKeyCode::Escape => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    None => {}
                },
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *control_flow = ControlFlow::Exit;
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    self.recreate_swapchain();
                    self.update();
                }
                Event::UserEvent(user_event) => match user_event {
                    RenderEvent::SampleCompleted => {
                        let start = std::time::Instant::now();
                        self.update();
                        println!(
                            "Update time (micro): {}",
                            (std::time::Instant::now() - start).as_micros()
                        );
                    }
                },
                Event::RedrawEventsCleared => {}
                _ => (),
            }
        });
    }

    fn update(&mut self) {
        match self.state.presentation_finished.as_mut() {
            Some(future) => future.cleanup_finished(),
            None => {}
        }
        self.state.presentation_finished = Some(sync::now(self.device.clone()).boxed());

        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain();
                    return;
                }
                Err(e) => {
                    panic!("Failed to acquire next image: {:?}", e)
                }
            };

        if suboptimal {
            self.recreate_swapchain();
        }

        let width = self.render_info.render_width;
        let height = self.render_info.render_height;

        // use compute to merge, tonemap and convert (copy from cpu swapchain)
        let layout = self
            .compute_pipeline
            .layout()
            .descriptor_set_layouts()
            .get(0)
            .unwrap();

        let image_view = ImageView::new(
            self.cpu_rendering.cpu_swapchain[!self
                .cpu_rendering
                .copy_to_first
                .load(std::sync::atomic::Ordering::Relaxed)
                as usize]
                .clone(),
        )
        .unwrap();

        let image_view_combined_buffer = ImageView::new(self.combined_buffer.clone()).unwrap();
        let set = PersistentDescriptorSet::new(
            layout.clone(),
            [{ WriteDescriptorSet::image_view(0, image_view) }, {
                WriteDescriptorSet::image_view(1, image_view_combined_buffer)
            }],
        )
        .unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.compute_pipeline.layout().clone(),
                0,
                set,
            )
            .dispatch([
                (self.render_info.render_width as f64 / 32.0).ceil() as u32,
                (self.render_info.render_height as f64 / 32.0).ceil() as u32,
                1,
            ])
            .unwrap();

        let compute_command_buffer = builder.build().unwrap();

        // blit to swapchain (copy + resize)
        let mut builder = AutoCommandBufferBuilder::primary(
            self.device.clone(),
            self.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        let extent: [u32; 2] = self.surface.window().inner_size().into();
        let sc_width = extent[0];
        let sc_height = extent[1];

        builder
            .blit_image(
                self.combined_buffer.clone(),
                [0, 0, 0],
                [width as i32, height as i32, 1],
                0,
                0,
                self.images[image_num].clone(),
                [0, 0, 0],
                [sc_width as i32, sc_height as i32, 1],
                0,
                0,
                1,
                Filter::Nearest,
            )
            .unwrap();

        let blit_command_buffer = builder.build().unwrap();

        // from cpu swapchain to combined image
        match &*self.cpu_rendering.to_sc.lock().unwrap() {
            Some(future) => {
                future.wait(None).unwrap();
            }
            None => {}
        }
        {
            let from_sc = &mut *self.cpu_rendering.from_sc.lock().unwrap();
            *from_sc = Some(
                match from_sc.take() {
                    Some(future) => future
                        .then_execute(self.queue.clone(), compute_command_buffer)
                        .unwrap()
                        .boxed_send_sync(),
                    None => sync::now(self.device.clone())
                        .then_execute(self.queue.clone(), compute_command_buffer)
                        .unwrap()
                        .boxed_send_sync(),
                }
                .then_signal_fence_and_flush()
                .unwrap(),
            );

            // copy to swapchain from combined image & present
            match from_sc {
                Some(val) => val.wait(None).unwrap(),
                None => {}
            }
        }
        let frame_future = self
            .state
            .presentation_finished
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), blit_command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), self.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        match frame_future {
            Ok(future) => {
                self.state.presentation_finished = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain();
                self.state.presentation_finished = Some(sync::now(self.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.state.presentation_finished = Some(sync::now(self.device.clone()).boxed());
            }
        }
    }
    fn recreate_swapchain(&mut self) {
        let dimensions: [u32; 2] = self.surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self.swapchain.recreate().dimensions(dimensions).build() {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        self.swapchain = new_swapchain;
        self.images = new_images;
    }
}
