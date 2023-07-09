mod gui;
mod rendering;

use implementations::SamplerProgress;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;

use {
	std::sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
	vulkano::{
		buffer::CpuAccessibleBuffer,
		command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer},
		device::{Device, Queue},
		image::StorageImage,
		sync::{self, GpuFuture},
	},
	winit::event_loop::EventLoopProxy,
};

pub use crate::gui::{Gui, RenderEvent};
pub use crate::rendering::Future;

pub struct Data {
	pub queue: Arc<Queue>,
	pub device: Arc<Device>,
	pub to_sc: rendering::Future,
	pub from_sc: rendering::Future,
	pub command_buffers: [Arc<PrimaryAutoCommandBuffer>; 2],
	pub buffer: Arc<CpuAccessibleBuffer<[f32]>>,
	pub sc_index: Arc<AtomicBool>,
	pub samples: Arc<AtomicU64>,
	pub total_samples: u64,
	pub rays_shot: Arc<AtomicU64>,
	pub event_proxy: EventLoopProxy<RenderEvent>,
	pub exit: Arc<AtomicBool>,
	pub bar: ProgressBar,
}

impl Data {
	pub fn new(
		queue: Arc<Queue>,
		device: Arc<Device>,
		to_sc: rendering::Future,
		from_sc: rendering::Future,
		command_buffers: [Arc<PrimaryAutoCommandBuffer>; 2],
		buffer: Arc<CpuAccessibleBuffer<[f32]>>,
		sc_index: Arc<AtomicBool>,
		samples: Arc<AtomicU64>,
		total_samples: u64,
		rays_shot: Arc<AtomicU64>,
		exit: Arc<AtomicBool>,
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
			total_samples,
			rays_shot,
			event_proxy,
			exit,
			bar: ProgressBar::new(total_samples).with_style(
				ProgressStyle::default_bar()
					.template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
					.unwrap(),
			),
		}
	}
}

pub fn create_command_buffers(
	device: Arc<Device>,
	queue: Arc<Queue>,
	buffer: Arc<CpuAccessibleBuffer<[f32]>>,
	sc: [Arc<StorageImage>; 2],
) -> [Arc<PrimaryAutoCommandBuffer>; 2] {
	let mut command_buffer_0 = None;
	let mut command_buffer_1 = None;
	for (i, sc_image) in sc.iter().enumerate() {
		let mut builder = AutoCommandBufferBuilder::primary(
			device.clone(),
			queue.family(),
			CommandBufferUsage::MultipleSubmit,
		)
		.unwrap();

		builder
			.copy_buffer_to_image(buffer.clone(), sc_image.clone())
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

pub fn sample_update(data: &mut Data, previous: &SamplerProgress, i: u64) -> bool {
	if data.exit.load(Ordering::Relaxed) {
		return true;
	}
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
				pres[0] += (acc[0] as f32 - pres[0]) / i as f32;
				pres[1] += (acc[1] as f32 - pres[1]) / i as f32;
				pres[2] += (acc[2] as f32 - pres[2]) / i as f32;
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

	data.bar.set_position(data.samples.load(Ordering::Relaxed));
	if data.samples.load(Ordering::Relaxed) == data.total_samples {
		data.bar.abandon()
	}

	// signal sample is ready to be presented
	data.event_proxy
		.send_event(RenderEvent::SampleCompleted)
		.is_err()
}
