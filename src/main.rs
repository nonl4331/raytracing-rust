use crate::parameters::Parameters;
use crate::scene::Scene;
use implementations::rt_core::*;
use implementations::*;

#[cfg(feature = "gui")]
use {
	gui::*,
	std::sync::{atomic::*, Arc},
	vulkano::{buffer::CpuAccessibleBuffer, instance::Instance},
	winit::event_loop::EventLoopProxy,
};

use crate::utility::*;

mod macros;
mod parameters;
mod scene;
mod utility;

#[cfg(feature = "gui")]
fn render_gui<M, P, C, S, A>(
	render_options: RenderOptions,
	filename: Option<String>,
	scene: Scene<M, P, C, S, A>,
) where
	M: Scatter + 'static,
	P: Primitive + 'static,
	C: Camera + 'static,
	S: NoHit + 'static,
	A: AccelerationStructure<Object = P, Material = M> + 'static,
{
	let required_extensions = vulkano_win::required_extensions();
	let instance = Instance::new(
		None,
		vulkano::instance::Version::V1_5,
		&required_extensions,
		None,
	)
	.unwrap();
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

		scene.render(
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
		println!("--------------------------------");

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
		println!("--------------------------------");

		save_file(
			filename,
			render_options.width,
			render_options.height,
			&*buffer.read().unwrap(),
			image_copy_finished,
		);
	}
}

fn render_tui<M, P, C, S, A>(
	render_options: RenderOptions,
	filename: Option<String>,
	scene: Scene<M, P, C, S, A>,
) where
	M: Scatter,
	P: Primitive,
	C: Camera,
	S: NoHit,
	A: AccelerationStructure<Object = P, Material = M>,
{
	let start = print_render_start(
		render_options.width,
		render_options.height,
		Some(render_options.samples_per_pixel),
	);

	let mut image = SamplerProgress::new(render_options.width * render_options.height, 3);
	let progress_bar_output = |sp: &mut SamplerProgress, previous: &SamplerProgress, i: u64| {
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

	scene.render(render_options, Some((&mut image, progress_bar_output)));

	let output = &image;

	let ray_count = output.rays_shot;

	print_final_statistics(start, ray_count, None);
	println!("--------------------------------");

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
}

fn main() {
	let (scene, parameters) = match parameters::process_args() {
		Some(data) => data,
		None => return,
	};

	let Parameters {
		render_options,
		gui,
		filename,
	} = parameters;

	if !gui {
		render_tui(render_options, filename, scene);
	} else {
		#[cfg(feature = "gui")]
		render_gui(render_options, filename, scene);
		#[cfg(not(feature = "gui"))]
		println!("feature: gui not enabled");
	}
}

#[cfg(feature = "gui")]
fn save_file(
	filename: Option<String>,
	width: u64,
	height: u64,
	buffer: &[f32],
	image_fence: Future,
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
