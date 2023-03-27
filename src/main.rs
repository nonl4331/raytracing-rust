use crate::parameters::Parameters;
use crate::scene::Scene;
use implementations::rt_core::*;
use implementations::*;
use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use output::*;

#[cfg(feature = "gui")]
use {
	gui::*,
	std::sync::{atomic::*, Arc},
	vulkano::{buffer::CpuAccessibleBuffer, instance::Instance},
	winit::event_loop::EventLoopProxy,
};

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
	S: NoHit<M> + 'static,
	A: AccelerationStructure<Object = P, Material = M, Sky = S> + 'static,
{
	let required_extensions = vulkano_win::required_extensions();
	let instance = Instance::new(
		None,
		vulkano::instance::Version::V1_5,
		&required_extensions,
		None,
	)
	.unwrap();
	let exit = Arc::new(AtomicBool::new(false));

	let gui = Gui::new(
		&instance,
		render_options.width as u32,
		render_options.height as u32,
		exit.clone(),
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
		exit,
		event_loop_proxy.unwrap(),
	);

	let start = print_render_start(
		render_options.width,
		render_options.height,
		render_options.gamma as f64,
		Some(render_options.samples_per_pixel),
	);

	let render_canceled = Arc::new(AtomicBool::new(true));

	let moved_render_canceled = render_canceled.clone();
	let moved_filename = filename.clone();

	let handle = std::thread::spawn(move || {
		let ray_count = data.rays_shot.clone();
		let samples = data.samples.clone();
		let buffer = data.buffer.clone();
		let to_sc = data.to_sc.clone();

		scene.render(
			render_options,
			Some((
				&mut data,
				|data: &mut Data, previous: &SamplerProgress, i: u64| -> bool {
					sample_update(data, previous, i)
				},
			)),
		);

		let ray_count = ray_count.load(Ordering::Relaxed);
		let samples = samples.load(Ordering::Relaxed);

		print_final_statistics(start, ray_count, samples);

		moved_render_canceled.store(false, Ordering::Relaxed);

		if let Some(filename) = moved_filename {
			match &*to_sc.lock().unwrap() {
				Some(future) => {
					future.wait(None).unwrap();
				}
				None => {}
			}

			save_data_to_image(
				filename,
				render_options.width as u32,
				render_options.height as u32,
				rgba_to_rgb(&*buffer.read().unwrap()),
				render_options.gamma,
			);
		}
	});

	gui.run();
	handle.join().unwrap();
}

fn render_tui<M, P, C, S, A>(
	render_options: RenderOptions,
	filename: Option<String>,
	scene: Scene<M, P, C, S, A>,
) where
	M: Scatter,
	P: Primitive,
	C: Camera,
	S: NoHit<M>,
	A: AccelerationStructure<Object = P, Material = M, Sky = S>,
{
	let start = print_render_start(
		render_options.width,
		render_options.height,
		render_options.gamma as f64,
		Some(render_options.samples_per_pixel),
	);

	struct Progress {
		pub sampler_progress: SamplerProgress,
		pub bar: ProgressBar,
	}

	let mut image = Progress {
		sampler_progress: SamplerProgress::new(render_options.width * render_options.height, 3),
		bar: ProgressBar::new(render_options.samples_per_pixel).with_style(
			ProgressStyle::default_bar()
				.template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
				.unwrap(),
		),
	};
	let progress_bar_output = |sp: &mut Progress, previous: &SamplerProgress, i: u64| -> bool {
		sp.sampler_progress.samples_completed += 1;
		sp.sampler_progress.rays_shot += previous.rays_shot;

		sp.sampler_progress
			.current_image
			.iter_mut()
			.zip(previous.current_image.iter())
			.for_each(|(pres, acc)| {
				*pres += (acc - *pres) / i as Float; // since copies first buffer when i=1
			});
		sp.bar.set_position(sp.sampler_progress.samples_completed);
		if sp.sampler_progress.samples_completed == render_options.samples_per_pixel {
			sp.bar.finish_and_clear()
		}
		false
	};

	scene.render(render_options, Some((&mut image, progress_bar_output)));

	let ray_count = image.sampler_progress.rays_shot;

	print_final_statistics(start, ray_count, image.sampler_progress.samples_completed);

	if let Some(filename) = filename {
		save_data_to_image(
			filename,
			render_options.width as u32,
			render_options.height as u32,
			image.sampler_progress.current_image,
			render_options.gamma,
		);
	}
}

fn main() {
	create_logger();
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
