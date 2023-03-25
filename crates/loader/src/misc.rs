use crate::Properties;
use crate::*;

use implementations::*;

impl Load for SimpleCamera {
	fn load(props: Properties, _: &mut Region) -> Result<(Option<String>, Self), LoadErr> {
		let origin = props.vec3("origin").unwrap_or(Vec3::new(3., 0., 0.));
		let lookat = props.vec3("lookat").unwrap_or(Vec3::zero());
		let vup = props.vec3("vup").unwrap_or(Vec3::new(0., 1., 0.));
		let fov = props.float("fov").unwrap_or(40.0);
		let aperture = props.float("aperture").unwrap_or(0.0);
		let focus = props.float("focus_dis").unwrap_or(10.0);

		let cam = Self::new(origin, lookat, vup, fov, 16.0 / 9.0, aperture, focus);
		Ok((None, cam))
	}
}

impl<T: Texture> Load for Sky<'_, T, AllMaterials<'_, T>> {
	fn load(props: Properties, region: &mut Region) -> Result<(Option<String>, Self), LoadErr> {
		let tex = props
			.texture("texture")
			.unwrap_or_else(|| props.default_texture());
		let res = props.vec2("sampler_res").unwrap_or(Vec2::new(100., 100.));

		let mat = AllMaterials::Emit(Emit::new(unsafe { &*(&*tex as *const _) }, 1.0));

		let mat = region.alloc(mat).shared();

		let sky = Self::new(
			unsafe { &*(&*tex as *const _) },
			unsafe { &*(&*mat as *const _) },
			(res.x as _, res.y as _),
		);
		Ok((None, sky))
	}
}
