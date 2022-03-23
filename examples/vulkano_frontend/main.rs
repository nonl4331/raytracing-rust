extern crate cpu_raytracer;
use cpu_raytracer::{
    image::camera::RandomSampler, material::MaterialEnum, texture::TextureEnum, *,
};

use crate::gui::{RenderEvent, GUI};

use vulkano::{
    buffer::CpuAccessibleBuffer,
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage},
    device::{Device, Queue},
    image::StorageImage,
    instance::Instance,
    sync::{self, FenceSignalFuture, GpuFuture},
    Version,
};
use winit::event_loop::EventLoopProxy;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};

const WIDTH: u32 = 2560;
const HEIGHT: u32 = 1440;

mod gui;
mod rendering;

struct Data {
    queue: Arc<Queue>,
    device: Arc<Device>,
    sc: [Arc<StorageImage>; 2],
    to_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
    from_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
    buffer: Arc<CpuAccessibleBuffer<[f32]>>,
    sc_index: Arc<AtomicBool>,
    samples: u64,
    rays_shot: u64,
    event_proxy: EventLoopProxy<RenderEvent>,
}

impl Data {
    pub fn new(
        queue: Arc<Queue>,
        device: Arc<Device>,
        sc: [Arc<StorageImage>; 2],
        to_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
        from_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
        buffer: Arc<CpuAccessibleBuffer<[f32]>>,
        sc_index: Arc<AtomicBool>,
        samples: u64,
        rays_shot: u64,
        event_proxy: EventLoopProxy<RenderEvent>,
    ) -> Self {
        Data {
            queue,
            device,
            sc,
            to_sc,
            from_sc,
            buffer,
            sc_index,
            samples,
            rays_shot,
            event_proxy,
        }
    }
}

fn main() {
    let required_extensions = vulkano_win::required_extensions();
    let instance = Instance::new(None, Version::V1_5, &required_extensions, None).unwrap();
    let gui = GUI::new(&instance);

    let event_loop_proxy: Option<EventLoopProxy<RenderEvent>> = if let Some(ref el) = gui.event_loop
    {
        Some(el.create_proxy())
    } else {
        None
    };
    let iter = [0.0 as Float, 0.0, 0.0, 0.0]
        .repeat((WIDTH * HEIGHT) as usize)
        .into_iter();
    let buffer = CpuAccessibleBuffer::from_iter(
        gui.device.clone(),
        vulkano::buffer::BufferUsage::all(),
        false,
        iter,
    )
    .unwrap();

    let data = Data::new(
        gui.queue.clone(),
        gui.device.clone(),
        gui.cpu_rendering.cpu_swapchain.clone(),
        gui.cpu_rendering.to_sc.clone(),
        gui.cpu_rendering.from_sc.clone(),
        buffer,
        gui.cpu_rendering.copy_to_first.clone(),
        0,
        0,
        event_loop_proxy.unwrap(),
    );

    std::thread::spawn(move || {
        let scene = get_scene();

        scene.generate_image_threaded(
            WIDTH as u64,
            HEIGHT as u64,
            1000,
            Some(
                |data: &mut Option<Data>, previous: &SamplerProgress, i: u64| {
                    sample_update(data, previous, i);
                },
            ),
            &mut Some(data),
        );
    });

    gui.run();
}

fn get_scene(
) -> Scene<PrimitiveEnum<MaterialEnum<TextureEnum>>, MaterialEnum<TextureEnum>, RandomSampler> {
    let mut primitives = Vec::new();

    let ground = sphere!(0, -1000, 0, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

    let glowy = sphere!(0, 0.5, 0, 0.5, &emit!(&solid_colour!(colour!(1)), 1.5));

    let cube = aacuboid!(
        -0.5,
        0.1,
        -0.5,
        -0.4,
        0.2,
        -0.4,
        &diffuse!(0.5, 0.5, 0.5, 0.5)
    );

    primitives.push(ground);
    primitives.push(glowy);
    primitives.push(cube);

    let camera = camera!(
        position!(-5, 3, -3),
        position!(0, 0.5, 0),
        position!(0, 1, 0),
        34,
        16.0 / 9.0,
        0,
        10
    );

    let bvh = bvh!(primitives, SplitType::Sah);

    scene!(camera, sky!(), random_sampler!(), bvh)
}

fn sample_update(data: &mut Option<Data>, previous: &SamplerProgress, i: u64) {
    // in this example data should always be Some(_)
    if let Some(data) = data {
        // update infomation about the rays shot and samples completed in the current render
        data.samples += 1;
        data.rays_shot += previous.rays_shot;

        // wait on from_sc future if is_some()
        match &*data.from_sc.lock().unwrap() {
            Some(future) => {
                future.wait(None).unwrap();
            }
            None => {}
        }
        match &*data.to_sc.lock().unwrap() {
            Some(future) => {
                future.wait(None).unwrap();
            }
            None => {}
        }

        {
            // get access to CpuAccessibleBuffer
            let mut buf = data.buffer.write().unwrap(); // jjhasfjhsajfhsakfksajf

            buf.chunks_mut(4)
                .zip(previous.current_image.chunks(3))
                .for_each(|(pres, acc)| {
                    pres[0] += (acc[0] - pres[0]) / i as Float;
                    pres[1] += (acc[1] - pres[1]) / i as Float;
                    pres[2] += (acc[2] - pres[2]) / i as Float;
                    pres[3] = 1.0;
                });
        }

        // copy to cpu swapchain
        let mut builder = AutoCommandBufferBuilder::primary(
            data.device.clone(),
            data.queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        )
        .unwrap();

        builder
            .copy_buffer_to_image(
                data.buffer.clone(),
                data.sc[data.sc_index.load(Ordering::Relaxed) as usize].clone(),
            )
            .unwrap();

        let command_buffer = builder.build().unwrap();

        // copy to swapchain and store op in to_sc future
        {
            let to_sc = &mut *data.to_sc.lock().unwrap();
            *to_sc = Some(
                match to_sc.take() {
                    Some(future) => future
                        .then_execute(data.queue.clone(), command_buffer)
                        .unwrap()
                        .boxed_send_sync(),
                    None => sync::now(data.device.clone())
                        .then_execute(data.queue.clone(), command_buffer)
                        .unwrap()
                        .boxed_send_sync(),
                }
                .then_signal_fence_and_flush()
                .unwrap(), // change to match
            );
        }

        // modify sc_index to !sc_index

        data.sc_index
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(!x))
            .unwrap();

        // signal sample is ready to be presented
        data.event_proxy
            .send_event(RenderEvent::SampleCompleted)
            .unwrap();
    } else {
        unreachable!();
    }
}
