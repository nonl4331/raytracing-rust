//------
// MISC
//------
#[macro_export]
macro_rules! position {
	($x:expr, $y:expr, $z:expr) => {
		rt_core::vec::Vec3::new(
			$x as rt_core::Float,
			$y as rt_core::Float,
			$z as rt_core::Float,
		)
	};
	($x:expr, $y:expr) => {
		rt_core::vec::Vec2::new($x as rt_core::Float, $y as rt_core::Float)
	};
}

#[macro_export]
macro_rules! colour {
	($r:expr,$g:expr,$b:expr) => {
		rt_core::Vec3::new(
			$r as rt_core::Float,
			$g as rt_core::Float,
			$b as rt_core::Float,
		)
	};
	($value:expr) => {
		rt_core::Vec3::new(
			$value as rt_core::Float,
			$value as rt_core::Float,
			$value as rt_core::Float,
		)
	};
}

#[macro_export]
macro_rules! rotation {
	($x:expr, $y:expr, $z:expr, D) => {
		rt_core::vec::Vec3::new(
			$x as rt_core::Float * rt_core::PI / 180.0,
			$y as rt_core::Float * rt_core::PI / 180.0,
			$z as rt_core::Float * rt_core::PI / 180.0,
		)
	};
	($x:expr, $y:expr, $z:expr, R) => {
		rt_core::vec::Vec3::new(
			$x as rt_core::Float,
			$y as rt_core::Float,
			$z as rt_core::Float,
		)
	};
}

#[macro_export]
macro_rules! axis {
	(X) => {
		implementations::Axis::X
	};
	(Y) => {
		implementations::Axis::Y
	};
	(Z) => {
		implementations::Axis::Z
	};
}

//-----
// TEXTURES
//-----
#[macro_export]
macro_rules! solid_colour {
	($r:expr, $g:expr, $b:expr) => {
		std::sync::Arc::new(implementations::AllTextures::SolidColour(
			implementations::SolidColour::new(colour!($r, $g, $b)),
		))
	};
	($colour:expr) => {
		std::sync::Arc::new(implementations::AllTextures::SolidColour(
			implementations::SolidColour::new($colour),
		))
	};
}

#[macro_export]
macro_rules! image {
	($filepath:expr) => {
		std::sync::Arc::new(implementations::AllTextures::ImageTexture(
			implementations::ImageTexture::new($filepath),
		))
	};
}

#[macro_export]
macro_rules! checkered {
	($colour_one:expr, $colour_two:expr) => {
		std::sync::Arc::new(implementations::AllTextures::CheckeredTexture(
			rt_core::CheckeredTexture::new($colour_one, $colour_two),
		))
	};
	($r1:expr, $g1:expr, $b1:expr, $r2:expr, $g2:expr, $b2:expr) => {
		std::sync::Arc::new(implementations::AllTextures::CheckeredTexture(
			rt_core::CheckeredTexture::new(colour!($r1, $g1, $b1), colour!($r2, $g2, $b2)),
		))
	};
}

#[macro_export]
macro_rules! texture_lerp {
	($colour_one:expr, $colour_two:expr) => {
		std::sync::Arc::new(implementations::AllTextures::Lerp(
			implementations::Lerp::new($colour_one, $colour_two),
		))
	};
	($r1:expr, $g1:expr, $b1:expr, $r2:expr, $g2:expr, $b2:expr) => {
		std::sync::Arc::new(implementations::AllTextures::Lerp(rt_core::Lerp::new(
			colour!($r1, $g1, $b1),
			colour!($r2, $g2, $b2),
		)))
	};
}

#[macro_export]
macro_rules! perlin {
	() => {
		std::sync::Arc::new(implementations::AllTextures::Perlin(Box::new(
			rt_core::Perlin::new(),
		)))
	};
}

//-----
// MATERIALS
//-----
#[macro_export]
macro_rules! diffuse {
	($r:expr,$g:expr,$b:expr, $absorption:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Lambertian(
			implementations::Lambertian::new(
				&std::sync::Arc::new(implementations::AllTextures::SolidColour(
					implementations::SolidColour::new(colour!($r, $g, $b)),
				)),
				$absorption as rt_core::Float,
			),
		))
	};
	($texture:expr,$absorption:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Lambertian(
			implementations::Lambertian::new($texture, $absorption as rt_core::Float),
		))
	};
}

#[macro_export]
macro_rules! reflect {
	($r:expr,$g:expr,$b:expr, $fuzz:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Reflect(
			rt_core::Reflect::new(
				&Arc::new(implementations::AllTextures::SolidColour(
					rt_core::SolidColour::new(colour!($r, $g, $b)),
				)),
				$fuzz as rt_core::Float,
			),
		));
	};
	($texture:expr,$fuzz:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Reflect(
			implementations::Reflect::new($texture, $fuzz as rt_core::Float),
		))
	};
}

#[macro_export]
macro_rules! refract {
	($r:expr,$g:expr,$b:expr, $eta:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Refract(
			rt_core::Refract::new(
				&std::sync::Arc::new(implementations::AllTextures::SolidColour(
					rt_core::SolidColour::new(colour!($r, $g, $b)),
				)),
				$eta as rt_core::Float,
			),
		))
	};
	($texture:expr,$eta:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Refract(
			implementations::Refract::new($texture, $eta as rt_core::Float),
		))
	};
}

#[macro_export]
macro_rules! emit {
	($r:expr,$g:expr,$b:expr, $strength:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Emit(rt_core::Emit::new(
			&std::sync::Arc::new(Texture::SolidColour(SolidColour::new(colour!($r, $g, $b)))),
			$strength as rt_core::Float,
		)));
	};
	($texture:expr,$strength:expr) => {
		std::sync::Arc::new(implementations::AllMaterials::Emit(
			implementations::Emit::new($texture, $strength as rt_core::Float),
		))
	};
}

//-----
// PRIMITIVES
//-----
#[macro_export]
macro_rules! sphere {
	($x:expr, $y:expr, $z:expr, $radius:expr, $material:expr) => {
		implementations::AllPrimitives::Sphere(implementations::sphere::Sphere::new(
			position!($x, $y, $z),
			$radius as Float,
			$material,
		))
	};
	($position:expr, $radius:expr, $material:expr) => {
		implementations::AllPrimitives::Sphere(implementations::sphere::Sphere::new(
			$position,
			$radius as rt_core::Float,
			$material,
		))
	};
}

#[macro_export]
macro_rules! aarect {
	($point_one:expr, $point_two:expr, $axis_value:expr, $axis:expr,  $material:expr) => {{
		let point_three = implementations::Axis::point_from_2d(
			rt_core::Vec2::new($point_one.x, $point_two.y),
			$axis,
			$axis_value,
		);
		let point_four = implementations::Axis::point_from_2d(
			rt_core::Vec2::new($point_two.x, $point_one.y),
			$axis,
			$axis_value,
		);
		let point_one = implementations::Axis::point_from_2d($point_one, $axis, $axis_value);
		let point_two = implementations::Axis::point_from_2d($point_two, $axis, $axis_value);
		let mut vec = Vec::new();
		vec.push(triangle!(point_one, point_two, point_three, $material));
		vec.push(triangle!(point_one, point_two, point_four, $material));

		vec
	}};
	($x1:expr, $y1:expr, $x2:expr, $y2:expr, $axis_value:expr, $axis:expr,  $material:expr) => {{
		let point_three =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x1, $y2), $axis, $axis_value);
		let point_four =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x2, $y1), $axis, $axis_value);
		let point_one =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x1, $y1), $axis, $axis_value);
		let point_two =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x2, $y2), $axis, $axis_value);

		let mut vec = Vec::new();
		vec.push(triangle!(point_one, point_two, point_three, $material));
		vec.push(triangle!(point_one, point_two, point_four, $material));

		vec
	}};
}

#[macro_export]
macro_rules! rect {
	($point_one:expr, $point_two:expr, $axis_value:expr, $axis:expr, $rotation:expr, $material:expr) => {{
		let center_point = 0.5 * ($point_one + $point_two);
		let point_three = implementations::Axis::point_from_2d(
			rt_core::Vec2::new($point_one.x, $point_two.y),
			$axis,
			$axis_value,
		);
		let point_four = implementations::Axis::point_from_2d(
			rt_core::Vec2::new($point_two.x, $point_one.y),
			$axis,
			$axis_value,
		);
		let point_one = implementations::Axis::point_from_2d($point_one, $axis, $axis_value);
		let point_two = implementations::Axis::point_from_2d($point_two, $axis, $axis_value);

		let sin_rot = rt_core::Vec3::new($rotation.x.sin(), $rotation.y.sin(), $rotation.z.sin());
		let cos_rot = rt_core::Vec3::new($rotation.x.cos(), $rotation.y.cos(), $rotation.z.cos());

		let point_one =
			$crate::utility::rotate_around_point(point_one, center_point, sin_rot, cos_rot);
		let point_two =
			$crate::utility::rotate_around_point(point_two, center_point, sin_rot, cos_rot);
		let point_three =
			$crate::utility::rotate_around_point(point_three, center_point, sin_rot, cos_rot);
		let point_four =
			$crate::utility::rotate_around_point(point_four, center_point, sin_rot, cos_rot);

		let vec = Vec::new();
		vec.push(triangle!(point_one, point_two, point_three, $material));
		vec.push(triangle!(point_one, point_two, point_four, $material));

		vec
	}};
	($x1:expr, $y1:expr, $x2:expr, $y2:expr, $axis_value:expr, $axis:expr, $rotation:expr, $material:expr) => {{
		let mut point_three =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x1, $y2), $axis, $axis_value);
		let mut point_four =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x2, $y1), $axis, $axis_value);
		let mut point_one =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x1, $y1), $axis, $axis_value);
		let mut point_two =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x2, $y2), $axis, $axis_value);

		let sin_rot = rt_core::Vec3::new($rotation.x.sin(), $rotation.y.sin(), $rotation.z.sin());
		let cos_rot = rt_core::Vec3::new($rotation.x.cos(), $rotation.y.cos(), $rotation.z.cos());

		$crate::utility::rotate_around_point(&mut point_one, center_point, sin_rot, cos_rot);

		$crate::utility::rotate_around_point(&mut point_two, center_point, sin_rot, cos_rot);

		$crate::utility::rotate_around_point(&mut point_three, center_point, sin_rot, cos_rot);

		$crate::utility::rotate_around_point(&mut point_four, center_point, sin_rot, cos_rot);

		let mut vec = Vec::new();
		vec.push(triangle!(point_one, point_two, point_three, $material));
		vec.push(triangle!(point_one, point_two, point_four, $material));

		vec
	}};
	($x1:expr, $y1:expr, $x2:expr, $y2:expr, $axis_value:expr, $axis:expr, $center_point:expr, $sin_rot:expr, $cos_rot:expr, $material:expr) => {{
		let mut point_three =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x1, $y2), $axis, $axis_value);
		let mut point_four =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x2, $y1), $axis, $axis_value);
		let mut point_one =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x1, $y1), $axis, $axis_value);
		let mut point_two =
			implementations::Axis::point_from_2d(&rt_core::Vec2::new($x2, $y2), $axis, $axis_value);

		$crate::utility::rotate_around_point(&mut point_one, $center_point, $sin_rot, $cos_rot);

		$crate::utility::rotate_around_point(&mut point_two, $center_point, $sin_rot, $cos_rot);

		$crate::utility::rotate_around_point(&mut point_three, $center_point, $sin_rot, $cos_rot);

		$crate::utility::rotate_around_point(&mut point_four, $center_point, $sin_rot, $cos_rot);

		let mut vec = Vec::new();
		vec.push(triangle!(point_one, point_two, point_three, $material));
		vec.push(triangle!(point_one, point_two, point_four, $material));

		vec
	}};
}

#[macro_export]
macro_rules! aacuboid {
	($point_one:expr, $point_two:expr, $material:expr) => {{
		let mut vec = Vec::new();
		vec.extend(aarect!(
			$point_one.x,
			$point_one.y,
			$point_two.x,
			$point_two.y,
			$point_one.z,
			implementations::Axis::Z,
			$material
		));
		vec.extend(aarect!(
			$point_one.x,
			$point_one.y,
			$point_two.x,
			$point_two.y,
			$point_two.z,
			implementations::Axis::Z,
			$material
		));

		vec.extend(aarect!(
			$point_one.x,
			$point_one.z,
			$point_two.x,
			$point_two.z,
			$point_one.y,
			implementations::Axis::Y,
			$material
		));
		vec.extend(aarect!(
			$point_one.x,
			$point_one.z,
			$point_two.x,
			$point_two.z,
			$point_two.y,
			implementations::Axis::Y,
			$material
		));

		vec.extend(aarect!(
			$point_one.y,
			$point_one.z,
			$point_two.y,
			$point_two.z,
			$point_one.x,
			implementations::Axis::X,
			$material
		));
		vec.extend(aarect!(
			$point_one.y,
			$point_one.z,
			$point_two.y,
			$point_two.z,
			$point_two.x,
			implementations::Axis::X,
			$material
		));
		vec
	}};
	($x1:expr, $y1:expr, $z1:expr, $x2:expr, $y2:expr, $z2:expr, $material:expr) => {{
		let mut vec = Vec::new();
		vec.extend(aarect!(
			$x1,
			$y1,
			$x2,
			$y2,
			$z1,
			&implementations::Axis::Z,
			$material
		));
		vec.extend(aarect!(
			$x1,
			$y1,
			$x2,
			$y2,
			$z2,
			&implementations::Axis::Z,
			$material
		));

		vec.extend(aarect!(
			$x1,
			$z1,
			$x2,
			$z2,
			$y1,
			&implementations::Axis::Y,
			$material
		));
		vec.extend(aarect!(
			$x1,
			$z1,
			$x2,
			$z2,
			$y2,
			&implementations::Axis::Y,
			$material
		));

		vec.extend(aarect!(
			$y1,
			$z1,
			$y2,
			$z2,
			$x1,
			&implementations::Axis::X,
			$material
		));
		vec.extend(aarect!(
			$y1,
			$z1,
			$y2,
			$z2,
			$x2,
			&implementations::Axis::X,
			$material
		));
		vec
	}};
}

#[macro_export]
macro_rules! cuboid {
	($point_one:expr, $point_two:expr, $rotation:expr, $material:expr) => {{
		let center_point = 0.5 * ($point_one + $point_two);
		let sin_rot = ($rotation.x.sin(), $rotation.y.sin(), $rotation.z.sin());
		let cos_rot = ($rotation.x.cos(), $rotation.y.cos(), $rotation.z.cos());

		let mut vec = Vec::new();
		vec.extend(rect!(
			$point_one.x,
			$point_one.y,
			$point_two.x,
			$point_two.y,
			$point_one.z,
			implementations::Axis::Z,
			$center_point,
			$sin_rot,
			$cos_rot,
			$material
		));
		vec.extend(rect!(
			$point_one.x,
			$point_one.y,
			$point_two.x,
			$point_two.y,
			$point_two.z,
			implementations::Axis::Z,
			$center_point,
			$sin_rot,
			$cos_rot,
			$material
		));

		vec.extend(rect!(
			$point_one.x,
			$point_one.z,
			$point_two.x,
			$point_two.z,
			$point_one.y,
			implementations::Axis::Y,
			$center_point,
			$sin_rot,
			$cos_rot,
			$material
		));
		vec.extend(rect!(
			$point_one.x,
			$point_one.z,
			$point_two.x,
			$point_two.z,
			$point_two.y,
			implementations::Axis::Y,
			$center_point,
			$sin_rot,
			$cos_rot,
			$material
		));

		vec.extend(rect!(
			$point_one.y,
			$point_one.z,
			$point_two.y,
			$point_two.z,
			$point_one.x,
			implementations::Axis::X,
			$center_point,
			$sin_rot,
			$cos_rot,
			$material
		));
		vec.extend(rect!(
			$point_one.y,
			$point_one.z,
			$point_two.y,
			$point_two.z,
			$point_two.x,
			implementations::Axis::X,
			$center_point,
			$sin_rot,
			$cos_rot,
			$material
		));
		vec
	}};
	($x1:expr, $y1:expr, $z1:expr, $x2:expr, $y2:expr, $z2:expr, $rotation:expr, $material:expr) => {{
		let point_one = rt_core::Vec3::new($x1, $y1, $z1);
		let point_two = rt_core::Vec3::new($x2, $y2, $z1);
		let center_point = 0.5 * (point_one + point_two);
		let sin_rot = rt_core::Vec3::new($rotation.x.sin(), $rotation.y.sin(), $rotation.z.sin());
		let cos_rot = rt_core::Vec3::new($rotation.x.cos(), $rotation.y.cos(), $rotation.z.cos());

		let mut vec = Vec::new();
		vec.extend(rect!(
			point_one.x,
			point_one.y,
			point_two.x,
			point_two.y,
			point_one.z,
			&implementations::Axis::Z,
			center_point,
			sin_rot,
			cos_rot,
			$material
		));
		vec.extend(rect!(
			point_one.x,
			point_one.y,
			point_two.x,
			point_two.y,
			point_two.z,
			&implementations::Axis::Z,
			center_point,
			sin_rot,
			cos_rot,
			$material
		));

		vec.extend(rect!(
			point_one.x,
			point_one.z,
			point_two.x,
			point_two.z,
			point_one.y,
			&implementations::Axis::Y,
			center_point,
			sin_rot,
			cos_rot,
			$material
		));
		vec.extend(rect!(
			point_one.x,
			point_one.z,
			point_two.x,
			point_two.z,
			point_two.y,
			&implementations::Axis::Y,
			center_point,
			sin_rot,
			cos_rot,
			$material
		));

		vec.extend(rect!(
			point_one.y,
			point_one.z,
			point_two.y,
			point_two.z,
			point_one.x,
			&implementations::Axis::X,
			center_point,
			sin_rot,
			cos_rot,
			$material
		));
		vec.extend(rect!(
			point_one.y,
			point_one.z,
			point_two.y,
			point_two.z,
			point_two.x,
			&implementations::Axis::X,
			center_point,
			sin_rot,
			cos_rot,
			$material
		));
		vec
	}};
}

#[macro_export]
macro_rules! model {
	($filepath:expr, $material:expr) => {
		$crate::ray_tracing::load_model::load_model($filepath, $material)
	};
}

// assumes orientation
#[macro_export]
macro_rules! triangle {
	($point_one:expr, $point_two:expr, $point_three:expr, $material:expr) => {{
		let normal = {
			let a = $point_two - $point_one;
			let b = $point_three - $point_one;
			a.cross(b)
		}
		.normalised();

		implementations::AllPrimitives::Triangle(implementations::Triangle::new(
			[$point_one, $point_two, $point_three],
			[normal; 3],
			$material,
		))
	}};

	($p1x:expr, $p1y:expr, $p1z:expr, $p2x:expr, $p2y:expr, $p2z:expr, $p3x:expr, $p3y:expr, $p3z:expr, $material:expr) => {{
		let point_one = position!($p1x, $p1y, $p1z);
		let point_two = position!($p2x, $p2y, $p2z);
		let point_three = position!($p3x, $p3y, $p3z);
		let normal = {
			let a = point_two - point_one;
			let b = point_three - point_one;
			a.cross(b)
		}
		.normalized();

		rt_core::AllPrimitives::Triangle(rt_core::Triangle::new_from_arc(
			[point_one, point_two, point_three],
			[normal; 3],
			$material,
		))
	}};
}

//-----
// OTHER STRUCTURES
//-----
#[macro_export]
macro_rules! camera {
	($origin:expr, $lookat:expr, $vup:expr, $fov:expr, $aspect_ratio:expr, $aperture:expr, $focus_dist:expr) => {
		std::sync::Arc::new(implementations::SimpleCamera::new(
			$origin,
			$lookat,
			$vup,
			$fov as rt_core::Float,
			$aspect_ratio as rt_core::Float,
			$aperture as rt_core::Float,
			$focus_dist as rt_core::Float,
		))
	};
}

#[macro_export]
macro_rules! random_sampler {
	() => {
		std::sync::Arc::new(implementations::random_sampler::RandomSampler {})
	};
}

#[macro_export]
macro_rules! sky {
	() => {
		std::sync::Arc::new(implementations::Sky::new(None))
	};
	($sky_texture:expr) => {
		std::sync::Arc::new(implementations::Sky::new(Some($sky_texture)))
	};
}

#[macro_export]
macro_rules! bvh {
	($primitives:expr, $split_type:expr) => {
		std::sync::Arc::new(implementations::Bvh::new($primitives, $split_type))
	};
}

#[macro_export]
macro_rules! scene {
	($camera:expr, $sky:expr, $sampler:expr, $bvh:expr) => {
		$crate::scene::Scene::new($camera, $sky, $sampler, $bvh)
	};
}
