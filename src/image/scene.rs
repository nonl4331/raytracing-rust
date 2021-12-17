use crate::acceleration::{bvh::Bvh, split::SplitType};
use crate::image::camera::{Camera, Sampler};
use crate::ray_tracing::{intersection::Primitive, material::Scatter, sky::Sky};
use crate::utility::math::Float;

use chrono::Local;

use std::{
    convert::TryInto,
    iter::FromIterator,
    marker::PhantomData,
    marker::{Send, Sync},
    sync::Arc,
    time::{Duration, Instant},
};

const SAMPLES_DEFAULT: u64 = 30;
const WIDTH_DEFAULT: u64 = 800;
const HEIGHT_DEFAULT: u64 = 600;
const FILENAME_DEFAULT: &str = "out.png";

pub struct Parameters {
    pub samples: u64,
    pub width: u64,
    pub height: u64,
    pub filename: String,
}

impl Parameters {
    pub fn new(
        samples: Option<u64>,
        width: Option<u64>,
        height: Option<u64>,
        filename: Option<String>,
    ) -> Self {
        Parameters {
            samples: samples.unwrap_or(SAMPLES_DEFAULT),
            width: width.unwrap_or(WIDTH_DEFAULT),
            height: height.unwrap_or(HEIGHT_DEFAULT),
            filename: filename.unwrap_or_else(|| FILENAME_DEFAULT.to_string()),
        }
    }
}

pub struct Scene<P: Primitive<M>, M: Scatter, S: Sampler> {
    pub primitives: Arc<Vec<P>>,
    pub bvh: Arc<Bvh>,
    pub camera: Arc<Camera>,
    pub sampler: Arc<S>,
    pub sky: Arc<Sky>,
    phantom: PhantomData<M>,
}
impl<P, M, S> Scene<P, M, S>
where
    P: Primitive<M> + Sync + Send + 'static,
    M: Scatter + Send + Sync,
    Vec<P>: FromIterator<P>,
    S: Sampler,
{
    pub fn new(
        camera: Arc<Camera>,
        sky: Arc<Sky>,
        sampler: Arc<S>,
        split_type: SplitType,
        primitives: Vec<P>,
    ) -> Self {
        let mut primitives: Vec<P> = primitives;

        let time = Local::now();

        println!("\n{} - Bvh construction started at", time.format("%X"));

        let start = Instant::now();
        let bvh = Arc::new(Bvh::new(&mut primitives, split_type));
        let end = Instant::now();
        let duration = end.checked_duration_since(start).unwrap();

        println!("\tBvh construction finished in: {}ms", duration.as_millis());
        println!("\tNumber of BVH nodes: {}\n", bvh.number_nodes());

        Scene {
            primitives: Arc::new(primitives),
            bvh,
            camera,
            sampler,
            sky,
            phantom: PhantomData,
        }
    }
    pub fn generate_image_threaded(&self, options: Parameters) {
        let progresses = self.sampler.sample_image(
            options.samples,
            options.width,
            options.height,
            self.camera.clone(),
            self.sky.clone(),
            self.primitives.clone(),
            self.bvh.clone(),
        );

        let mut exit = false;
        while !exit {
            let mut samples_sum = 0;
            for progress in &progresses {
                samples_sum += progress.read().unwrap().samples_completed;
            }
            println!("{} / {} samples completed", samples_sum, options.samples);
            if samples_sum == options.samples {
                exit = true;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        let progresses: Vec<Vec<Float>> = progresses
            .iter()
            .map(|prog| prog.read().unwrap().current_image.clone())
            .collect();

        let image = progresses.iter().fold(
            vec![0.0; (options.width * options.height * 3) as usize],
            |acc, image| acc.iter().zip(image).map(|(&a, &b)| a + b).collect(),
        );

        let image: Vec<u8> = image
            .iter()
            .map(|value| (value.sqrt() * 255.0) as u8)
            .collect();

        image::save_buffer(
            options.filename,
            &image,
            options.width.try_into().unwrap(),
            options.height.try_into().unwrap(),
            image::ColorType::Rgb8,
        )
        .unwrap();
    }
}

pub fn line_break() {
    println!("------------------------------");
}

fn get_readable_duration(duration: Duration) -> String {
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
