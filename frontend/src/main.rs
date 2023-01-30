#[cfg(feature = "gui")]
use {
	gui::{Gui, RenderEvent},
	std::sync::{
		atomic::{AtomicBool, AtomicU64, Ordering},
		Arc,
	},
	vulkano::{
		buffer::CpuAccessibleBuffer,
		command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryAutoCommandBuffer},
		device::{Device, Queue},
		image::StorageImage,
		instance::Instance,
		sync::{self, GpuFuture},
		Version,
	},
	winit::event_loop::EventLoopProxy,
};

//use scene::rt_core::{Float, SamplerProgress};
use std::env;

#[cfg(feature = "gui")]
mod gui;
#[cfg(feature = "gui")]
mod rendering;

//mod generate;
//mod load_model;
mod macros;
//mod parameters;
mod scene;
mod utility;

#[cfg(feature = "gui")]
struct Data {
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
	event_proxy: EventLoopProxy<RenderEvent>,
}

#[cfg(feature = "gui")]
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
		}
	}
}

fn main() {
	let _args: Vec<String> = env::args().collect();

	/*if let Some((scene, parameters)) = parameters::process_args(args) {
		let (render_options, filename) = (parameters.render_options, parameters.filename.clone());
		if !parameters.gui {
			let start = print_render_start(
				render_options.width,
				render_options.height,
				Some(render_options.samples_per_pixel),
			);

			let mut image = SamplerProgress::new(render_options.width * render_options.height, 3);
			let progress_bar_output =
				|sp: &mut SamplerProgress, previous: &SamplerProgress, i: u64| {
					sp.samples_completed += 1;
					sp.rays_shot += previous.rays_shot;

					sp.current_image
						.iter_mut()
						.zip(previous.current_image.iter())
						.for_each(|(pres, acc)| {
							*pres += (acc - *pres) / i as Float; // since copies first buffer when i=1
						});

					get_progress_output(sp.samples_completed, render_options.samples_per_pixel);
				};

			scene.generate_image_threaded(render_options, Some((&mut image, progress_bar_output)));

			let output = &image;

			let ray_count = output.rays_shot;

			print_final_statistics(start, ray_count, None);
			line_break();

			let output: Vec<u8> = output
				.current_image
				.iter()
				.map(|val| (val.sqrt() * 255.999) as u8)
				.collect();

			if let Some(filename) = filename {
				save_u8_to_image(
					render_options.width,
					render_options.height,
					output,
					filename,
					false,
				);
			}
		} else {
			#[cfg(feature = "gui")]
			{
				let required_extensions = vulkano_win::required_extensions();
				let instance =
					Instance::new(None, Version::V1_5, &required_extensions, None).unwrap();
				let gui = Gui::new(
					&instance,
					render_options.width as u32,
					render_options.height as u32,
				);

				let event_loop_proxy: Option<EventLoopProxy<RenderEvent>> =
					gui.event_loop.as_ref().map(|el| el.create_proxy());
				let iter = [0.0f32, 0.0, 0.0, 0.0]
					.repeat((render_options.width * render_options.height) as usize)
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

				let mut data = Data::new(
					gui.queue.clone(),
					gui.device.clone(),
					gui.cpu_rendering.to_sc.clone(),
					gui.cpu_rendering.from_sc.clone(),
					command_buffers,
					buffer.clone(),
					gui.cpu_rendering.copy_to_first.clone(),
					samples.clone(),
					render_options.samples_per_pixel,
					ray_count.clone(),
					event_loop_proxy.unwrap(),
				);

				let image_copy_finished = data.to_sc.clone();

				let start = print_render_start(render_options.width, render_options.height, None);

				let render_canceled = Arc::new(AtomicBool::new(true));

				let moved_render_canceled = render_canceled.clone();
				let moved_filename = filename.clone();

				std::thread::spawn(move || {
					let ray_count = data.rays_shot.clone();
					let samples = data.samples.clone();
					let buffer = data.buffer.clone();
					let to_sc = data.to_sc.clone();

					scene.generate_image_threaded(
						render_options,
						Some((
							&mut data,
							|data: &mut Data, previous: &SamplerProgress, i: u64| {
								sample_update(data, previous, i);
							},
						)),
					);

					let ray_count = ray_count.load(Ordering::Relaxed);
					let samples = samples.load(Ordering::Relaxed);

					print_final_statistics(start, ray_count, Some(samples));
					line_break();

					moved_render_canceled.store(false, Ordering::Relaxed);

					save_file(
						moved_filename,
						render_options.width,
						render_options.height,
						&*buffer.read().unwrap(),
						to_sc,
					);
				});

				gui.run();
				if render_canceled.load(Ordering::Relaxed) {
					let ray_count = ray_count.load(Ordering::Relaxed);
					let samples = samples.load(Ordering::Relaxed);

					print_final_statistics(start, ray_count, Some(samples));
					line_break();

					save_file(
						filename,
						render_options.width,
						render_options.height,
						&*buffer.read().unwrap(),
						image_copy_finished,
					);
				}
			}
			#[cfg(not(feature = "gui"))]
			println!("feature: gui not enabled");
		}
	}*/
}

#[cfg(feature = "gui")]
fn create_command_buffers(
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

#[cfg(feature = "gui")]
fn sample_update(data: &mut Data, previous: &SamplerProgress, i: u64) {
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

	get_progress_output(i, data.total_samples);

	// signal sample is ready to be presented
	data.event_proxy
		.send_event(RenderEvent::SampleCompleted)
		.unwrap();
}

#[cfg(feature = "gui")]
fn save_file(
	filename: Option<String>,
	width: u64,
	height: u64,
	buffer: &[f32],
	image_fence: rendering::Future,
) {
	match filename {
		Some(filename) => {
			match &*image_fence.lock().unwrap() {
				Some(future) => {
					future.wait(None).unwrap();
				}
				None => {}
			}

			save_u8_to_image(
				width,
				height,
				buffer.iter().map(|val| (val * 255.999) as u8).collect(),
				filename,
				true,
			)
		}
		None => {}
	}
}
