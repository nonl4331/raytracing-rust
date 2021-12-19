use crate::acceleration::bvh::Bvh;
use crate::image::camera::{Camera, Sampler, SamplerProgress};
use crate::ray_tracing::{intersection::Primitive, material::Scatter, sky::Sky};

use std::{
    iter::FromIterator,
    marker::{Send, Sync},
    sync::{Arc, RwLock},
};

pub struct Scene<P: Primitive<M>, M: Scatter, S: Sampler> {
    pub bvh: Arc<Bvh<P, M>>,
    pub camera: Arc<Camera>,
    pub sampler: Arc<S>,
    pub sky: Arc<Sky>,
}
impl<P, M: 'static, S> Scene<P, M, S>
where
    P: Primitive<M> + Sync + Send + 'static,
    M: Scatter + Send + Sync,
    Vec<P>: FromIterator<P>,
    S: Sampler,
{
    pub fn new(camera: Arc<Camera>, sky: Arc<Sky>, sampler: Arc<S>, bvh: Arc<Bvh<P, M>>) -> Self {
        Scene {
            bvh,
            camera,
            sampler,
            sky,
        }
    }
    pub fn generate_image_threaded(
        &self,
        width: u64,
        height: u64,
        samples: u64,
    ) -> Vec<Arc<RwLock<SamplerProgress>>> {
        self.sampler.sample_image(
            samples,
            width,
            height,
            self.camera.clone(),
            self.sky.clone(),
            self.bvh.clone(),
        )
    }
}
