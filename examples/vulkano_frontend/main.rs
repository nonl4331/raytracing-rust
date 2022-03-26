extern crate cpu_raytracer;
use cpu_raytracer::{
	image::camera::RandomSampler, material::MaterialEnum, texture::TextureEnum, *,
};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;

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

use chrono::Local;
use std::sync::{
	atomic::{AtomicBool, AtomicU64, Ordering},
	Arc, Mutex,
};
use std::time::{Duration, Instant};

const WIDTH: u32 = 2560;
const HEIGHT: u32 = 1440;

mod gui;
mod rendering;

struct Data {
	queue: Arc<Queue>,
	device: Arc<Device>,
	to_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
	from_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
	command_buffers: [Arc<PrimaryAutoCommandBuffer>; 2],
	buffer: Arc<CpuAccessibleBuffer<[f32]>>,
	sc_index: Arc<AtomicBool>,
	samples: Arc<AtomicU64>,
	rays_shot: Arc<AtomicU64>,
	event_proxy: EventLoopProxy<RenderEvent>,
}

impl Data {
	pub fn new(
		queue: Arc<Queue>,
		device: Arc<Device>,
		to_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
		from_sc: Arc<Mutex<Option<FenceSignalFuture<Box<dyn GpuFuture + Send + Sync + 'static>>>>>,
		command_buffers: [Arc<PrimaryAutoCommandBuffer>; 2],
		buffer: Arc<CpuAccessibleBuffer<[f32]>>,
		sc_index: Arc<AtomicBool>,
		samples: Arc<AtomicU64>,
		rays_shot: Arc<AtomicU64>,
		event_proxy: EventLoopProxy<RenderEvent>,
	) -> Self {
		Data {
			queue,
			device,
			to_sc,
			from_sc,
			command_buffers,
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
		true,
		iter,
	)
	.unwrap();

	let samples = Arc::new(AtomicU64::new(0));
	let ray_count = Arc::new(AtomicU64::new(0));

	let command_buffers = create_command_buffers(
		gui.device.clone(),
		gui.queue.clone(),
		buffer.clone(),
		gui.cpu_rendering.cpu_swapchain.clone(),
	);

	let data = Data::new(
		gui.queue.clone(),
		gui.device.clone(),
		gui.cpu_rendering.to_sc.clone(),
		gui.cpu_rendering.from_sc.clone(),
		command_buffers,
		buffer,
		gui.cpu_rendering.copy_to_first.clone(),
		samples.clone(),
		ray_count.clone(),
		event_loop_proxy.unwrap(),
	);

	let start = Instant::now();
	let time = Local::now();
	println!("{} - Render started", time.format("%X"));
	println!("\tWidth: {}", WIDTH);
	println!("\tHeight: {}", HEIGHT);
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

	let ray_count = ray_count.load(Ordering::Relaxed);

	let end = Instant::now();
	let duration = end.checked_duration_since(start).unwrap();
	let time = Local::now();
	println!(
		"\u{001b}[2K\r{} - Finised rendering image",
		time.format("%X")
	);
	println!("\tRender Time: {}", get_readable_duration(duration));
	println!("\tRays: {}", ray_count);
	println!("\tSamples: {}", samples.load(Ordering::Relaxed));
	println!(
		"\tMrays/s: {:.2}",
		(ray_count as f64 / duration.as_secs_f64()) / 1000000.0
	);
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

fn create_command_buffers(
	device: Arc<Device>,
	queue: Arc<Queue>,
	buffer: Arc<CpuAccessibleBuffer<[f32]>>,
	sc: [Arc<StorageImage>; 2],
) -> [Arc<PrimaryAutoCommandBuffer>; 2] {
	let mut command_buffer_0 = None;
	let mut command_buffer_1 = None;
	for i in 0..2 {
		let mut builder = AutoCommandBufferBuilder::primary(
			device.clone(),
			queue.family(),
			CommandBufferUsage::MultipleSubmit,
		)
		.unwrap();

		builder
			.copy_buffer_to_image(buffer.clone(), sc[i].clone())
			.unwrap();
		if i == 0 {
			command_buffer_0 = Some(builder.build().unwrap());
		} else {
			command_buffer_1 = Some(builder.build().unwrap());
		}
	}

	[
		Arc::new(command_buffer_0.unwrap()),
		Arc::new(command_buffer_1.unwrap()),
	]
}

fn sample_update(data: &mut Option<Data>, previous: &SamplerProgress, i: u64) {
	// in this example data should always be Some(_)
	if let Some(data) = data {
		// update infomation about the rays shot and samples completed in the current render
		data.samples.fetch_add(1, Ordering::Relaxed);
		data.rays_shot
			.fetch_add(previous.rays_shot, Ordering::Relaxed);

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
			let mut buf = data.buffer.write().unwrap();
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
		let command_buffer =
			data.command_buffers[data.sc_index.load(Ordering::Relaxed) as usize].clone();

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

pub fn get_readable_duration(duration: Duration) -> String {
	let days = duration.as_secs() / 86400;

	let days_string = match days {
		0 => "".to_string(),
		1 => format!("{} day, ", days),
		_ => format!("{} days, ", days),
	};

	let hours = (duration.as_secs() - days * 86400) / 3600;
	let hours_string = match hours {
		0 => "".to_string(),
		1 => format!("{} hour, ", hours),
		_ => format!("{} hours, ", hours),
	};

	let minutes = (duration.as_secs() - days * 86400 - hours * 3600) / 60;
	let minutes_string = match minutes {
		0 => "".to_string(),
		1 => format!("{} minute, ", minutes),
		_ => format!("{} minutes, ", minutes),
	};

	let seconds = duration.as_secs() % 60;
	let seconds_string = match seconds {
		0 => "~0 seconds".to_string(),
		1 => format!("{} second", seconds),
		_ => format!("{} seconds", seconds),
	};
	days_string + &hours_string + &minutes_string + &seconds_string
}
