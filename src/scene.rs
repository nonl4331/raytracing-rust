use implementations::random_sampler::RandomSampler;
use implementations::rt_core::*;
use implementations::*;
use region::Region;
use std::mem::ManuallyDrop;

pub struct Scene<M, P, C, S, A>
where
	M: Scatter,
	P: Primitive,
	C: Camera,
	S: NoHit<M>,
	A: AccelerationStructure<Object = P, Material = M, Sky = S>,
{
	acceleration: A,
	camera: C,
	_region: ManuallyDrop<Region>,
}

impl<M, P, C, S, A> Scene<M, P, C, S, A>
where
	M: Scatter,
	P: Primitive,
	C: Camera,
	S: NoHit<M>,
	A: AccelerationStructure<Object = P, Material = M, Sky = S>,
{
	pub fn new(acceleration: A, camera: C, region: ManuallyDrop<Region>) -> Self {
		Self {
			acceleration,
			camera,
			_region: region,
		}
	}
	pub fn render<T>(
		&self,
		opts: RenderOptions,
		update: Option<(&mut T, impl Fn(&mut T, &SamplerProgress, u64) -> bool)>,
	) {
		let sampler = RandomSampler {};
		sampler.sample_image(opts, &self.camera, &self.acceleration, update);
	}
}

unsafe impl<M, P, C, A, S> Send for Scene<M, P, C, S, A>
where
	M: Scatter,
	P: Primitive,
	C: Camera,
	S: NoHit<M>,
	A: AccelerationStructure<Object = P, Material = M, Sky = S>,
{
}

/*#[cfg(test)]
mod tests {
	use super::*;
	use loader::load_str_full;

	const DATA: &str = "camera (
	origin   -5 3 -3
	lookat   0 0.5 0
	vup      0 1 0
	fov      34.0
	aperture 0.0
	focus_dis 10.0
)

texture sky (
	type solid
	colour 0.0
)

sky (
	texture sky
)

texture grey (
	type solid
	colour 0.5
)

texture white (
	type solid
	colour 1.0
)

material ground (
	type lambertian
	texture grey
	albedo 0.5
)

material light (
	type emissive
	texture white
	strength 1.5
)

primitive (
	type sphere
	material ground
	centre 0 -1000 0
	radius 1000
)

primitive (
	type sphere
	material light
	centre 0 0.5 0
	radius 0.5
)

primitive (
	type sphere
	material ground
	centre -0.45 0.15 -0.45
	radius 0.05
)";

	#[test]
	fn scene() {
		let mut region = Region::new();
		type Tex = AllTextures;
		type Mat<'a> = AllMaterials<'a, Tex>;
		type Prim<'a> = AllPrimitives<'a, Mat<'a>>;
		type SkyType<'a> = Sky<'a, Tex, Mat<'a>>;
		type SceneType<'a> = Scene<
			Mat<'a>,
			Prim<'a>,
			SimpleCamera,
			SkyType<'a>,
			Bvh<Prim<'a>, Mat<'a>, SkyType<'a>>,
		>;
		let stuff =
			load_str_full::<Tex, Mat, Prim, SimpleCamera, SkyType>(&mut region, DATA).unwrap();

		let (p, camera, sky) = stuff;
		let bvh: Bvh<Prim, Mat, SkyType> = Bvh::new(p, sky.clone(), split::SplitType::Sah);

		let scene: SceneType<'static> = Scene::new(bvh, camera, sky, region);

		scene.render::<()>(
			RenderOptions {
				samples_per_pixel: 1,
				render_method: RenderMethod::MIS,
				width: 1920,
				height: 1080,
			},
			None as Option<(&mut (), fn(&mut (), &SamplerProgress, u64) -> bool)>,
		);
	}
}*/
