use crate::ray_tracing::{
	intersection::{Hit, Primitive, SurfaceIntersection},
	material::Scatter,
	primitives::Axis,
	ray::Ray,
};
use crate::utility::vec::Vec3;
use crate::{
	acceleration::{
		aabb::Aabb,
		split::{Split, SplitType},
	},
	utility::math::sort_by_indices,
};
use std::collections::VecDeque;
use std::marker::PhantomData;

#[cfg(all(feature = "f64"))]
use std::f64::EPSILON;

#[cfg(not(feature = "f64"))]
use std::f32::EPSILON;

#[derive(Debug, Clone, Copy)]
pub struct PrimitiveInfo {
	pub index: usize,
	pub min: Vec3,
	pub max: Vec3,
	pub center: Vec3,
}

pub trait PrimitiveSampling<P, M>: AccelerationStructure<M>
where
	P: Primitive<M>,
	M: Scatter,
{
	fn sample_object(&self, hit: &Hit, index: usize) -> (Vec3, Option<Vec3>, Vec3);

	fn get_samplable(&self) -> &[usize];

	fn get_object(&self, index: usize) -> Option<&P>;
}

pub trait AccelerationStructure<M: Scatter> {
	fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)>;

	fn check_hit_index(&self, ray: &Ray, object_index: usize) -> Option<SurfaceIntersection<M>>;

	fn check_hit(&self, ray: &Ray) -> Option<(SurfaceIntersection<M>, usize)>;

	fn number_nodes(&self) -> usize;
}

impl PrimitiveInfo {
	fn new<P: Primitive<M>, M: Scatter>(index: usize, primitive: &P) -> PrimitiveInfo {
		let aabb = primitive.get_aabb().unwrap();
		let min = aabb.min;
		let max = aabb.max;
		PrimitiveInfo {
			index,
			min,
			max,
			center: 0.5 * (min + max),
		}
	}
}

pub struct Bvh<P: Primitive<M>, M: Scatter> {
	split_type: SplitType,
	nodes: Vec<Node>,
	pub primitives: Vec<P>,
	pub lights: Vec<usize>,
	phantom: PhantomData<M>,
}

impl<P, M> Bvh<P, M>
where
	P: Primitive<M>,
	M: Scatter,
{
	pub fn new(primitives: Vec<P>, split_type: SplitType) -> Self {
		let mut bvh = Self {
			split_type,
			nodes: Vec::new(),
			primitives,
			lights: Vec::new(),
			phantom: PhantomData,
		};
		let mut primitives_info: Vec<PrimitiveInfo> = bvh
			.primitives
			.iter()
			.enumerate()
			.map(|(index, primitive)| PrimitiveInfo::new(index, primitive))
			.collect();

		bvh.build_bvh(&mut Vec::new(), 0, &mut primitives_info);

		sort_by_indices(
			&mut bvh.primitives,
			primitives_info.iter().map(|&info| info.index).collect(),
		);

		for (i, prim) in bvh.primitives.iter().enumerate() {
			if prim.material_is_light() {
				bvh.lights.push(i);
			}
		}

		bvh
	}

	fn build_bvh(
		&mut self,
		ordered_primitives: &mut Vec<usize>,
		offset: usize,
		primitives_info: &mut [PrimitiveInfo],
	) -> usize {
		let number_primitives = primitives_info.len();

		let mut bounds = None;
		for info in primitives_info.iter() {
			Aabb::merge(&mut bounds, Aabb::new(info.min, info.max));
		}

		let mut children = None;

		let node_index = self.nodes.len();

		self.nodes
			.push(Node::new(bounds.unwrap(), offset, number_primitives));

		if number_primitives == 1 {
			ordered_primitives.push(primitives_info[0].index);
		} else {
			let mut center_bounds = None;
			for info in primitives_info[0..number_primitives].iter() {
				Aabb::extend_contains(&mut center_bounds, info.center);
			}

			let center_bounds = center_bounds.unwrap();

			let axis = Axis::get_max_axis(&center_bounds.get_extent());

			if (axis.get_axis_value(center_bounds.min) - axis.get_axis_value(center_bounds.max))
				.abs() < 100.0 * EPSILON
			{
				for primitive in primitives_info {
					ordered_primitives.push(primitive.index);
				}
			} else {
				let mid =
					self.split_type
						.split(&bounds.unwrap(), &center_bounds, &axis, primitives_info);
				if mid != 0 {
					let (left, right) = primitives_info.split_at_mut(mid);

					children = Some((
						self.build_bvh(ordered_primitives, offset, left),
						self.build_bvh(ordered_primitives, offset + left.len(), right),
					));
				} else {
					for primitive in primitives_info {
						ordered_primitives.push(primitive.index);
					}
				}
			}
		}

		if let Some(children) = children {
			self.nodes[node_index].set_child(children.0, 0);
			self.nodes[node_index].set_child(children.1, 1);
		}

		node_index
	}

	pub fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)> {
		let mut offset_len = Vec::new();

		let mut node_stack = VecDeque::new();
		node_stack.push_back(0);
		while !node_stack.is_empty() {
			let index = node_stack.pop_front().unwrap();

			let node = &self.nodes[index];

			if !node.bounds.does_int(ray) {
				continue;
			}

			match node.children {
				Some(children) => {
					node_stack.push_back(children[0]);
					node_stack.push_back(children[1]);
				}
				None => {
					offset_len.push((node.primitive_offset, node.number_primitives));
				}
			}
		}
		offset_len
	}
	pub fn number_nodes(&self) -> usize {
		self.nodes.len()
	}
}

impl<M, P> AccelerationStructure<M> for Bvh<P, M>
where
	P: Primitive<M>,
	M: Scatter,
{
	fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)> {
		let mut offset_len = Vec::new();

		let mut node_stack = VecDeque::new();
		node_stack.push_back(0);
		while !node_stack.is_empty() {
			let index = node_stack.pop_front().unwrap();

			let node = &self.nodes[index];

			if !node.bounds.does_int(ray) {
				continue;
			}

			match node.children {
				Some(children) => {
					node_stack.push_back(children[0]);
					node_stack.push_back(children[1]);
				}
				None => {
					offset_len.push((node.primitive_offset, node.number_primitives));
				}
			}
		}
		offset_len
	}

	fn check_hit_index(&self, ray: &Ray, index: usize) -> Option<SurfaceIntersection<M>> {
		let object = &self.primitives[index];

		let offset_lens = self.get_intersection_candidates(&ray);

		let intersection = object.get_int(&ray);

		let light_t = match intersection {
			Some(ref hit) => {
				if hit.hit.t > 0.0 {
					hit.hit.t
				} else {
					return None;
				}
			}
			None => return None,
		};

		// check if object blocking
		for offset_len in offset_lens {
			let offset = offset_len.0;
			let len = offset_len.1;
			for current_index in offset..(offset + len) {
				if current_index == index {
					continue;
				}
				let tobject = &self.primitives[current_index];
				// check for hit
				if let Some(current_hit) = tobject.get_int(&ray) {
					// make sure ray is going forwards
					if current_hit.hit.t > 0.0 && current_hit.hit.t < light_t {
						return None;
					}
				}
			}
		}
		intersection
	}

	fn check_hit(&self, ray: &Ray) -> Option<(SurfaceIntersection<M>, usize)> {
		let offset_lens = self.get_intersection_candidates(ray);

		let mut hit: Option<(SurfaceIntersection<M>, usize)> = None;

		for offset_len in offset_lens {
			let offset = offset_len.0;
			let len = offset_len.1;
			for index in offset..(offset + len) {
				let object = &self.primitives[index];
				// check for hit
				if let Some(current_hit) = object.get_int(ray) {
					// make sure ray is going forwards
					if current_hit.hit.t > 0.0 {
						// check if hit already exists
						if let Some((last_hit, _)) = &hit {
							// check if t value is close to 0 than previous hit
							if current_hit.hit.t < last_hit.hit.t {
								hit = Some((current_hit, index));
							}
							continue;
						}

						// if hit doesn't exist set current hit to hit
						hit = Some((current_hit, index));
					}
				}
			}
		}
		hit
	}
	fn number_nodes(&self) -> usize {
		self.nodes.len()
	}
}

impl<P, M> PrimitiveSampling<P, M> for Bvh<P, M>
where
	P: Primitive<M>,
	M: Scatter,
{
	fn sample_object(&self, hit: &Hit, index: usize) -> (Vec3, Option<Vec3>, Vec3) {
		let object = &self.primitives[index];
		let (object_point, dir, _normal) = object.sample_visible_from_point(hit.point);

		let ray = Ray::new(hit.point, dir, 0.0);

		let li = match self.check_hit_index(&ray, index) {
			Some(int) => Some(int.material.get_emission(hit)),
			None => return (Vec3::zero(), None, Vec3::zero()),
		};

		(dir, li, object_point)
	}
	fn get_samplable(&self) -> &[usize] {
		&self.lights
	}
	fn get_object(&self, index: usize) -> Option<&P> {
		self.primitives.get(index)
	}
}

#[derive(Debug)]
pub struct Node {
	bounds: Aabb,
	children: Option<[usize; 2]>,
	primitive_offset: usize,
	number_primitives: usize,
}

impl Node {
	fn new(bounds: Aabb, primitive_offset: usize, number_primitives: usize) -> Self {
		Node {
			bounds,
			children: None,
			primitive_offset,
			number_primitives,
		}
	}
	fn set_child(&mut self, child_index: usize, index: usize) {
		match self.children {
			Some(_) => {
				let mut val = self.children.unwrap();
				val[index] = child_index;
				self.children = Some(val);
			}
			None => {
				let mut children = [0, 0];
				children[index] = child_index;
				self.children = Some(children);
			}
		}
	}
}

#[cfg(test)]
mod tests {

	use crate::acceleration::bvh::PrimitiveInfo;
	use crate::material::MaterialEnum;
	use crate::ray_tracing::{intersection::Primitive, primitives::PrimitiveEnum};
	use crate::texture::TextureEnum;
	use crate::utility::{math::Float, vec::Vec3};
	use crate::*;
	use rand::{distributions::Alphanumeric, rngs::SmallRng, thread_rng, Rng, SeedableRng};
	use rand_seeder::Seeder;

	#[test]
	fn primitive_info_new() {
		let sphere = sphere!(colour!(1), 0.2, &refract!(1, 1, 1, 1.5));
		let info = PrimitiveInfo::new(3, &sphere);
		assert!(
			info.max == 1.2 * Vec3::one()
				&& info.min == 0.8 * Vec3::one()
				&& info.center == Vec3::one()
				&& info.index == 3
		);
	}

	#[test]
	fn node_containment() {
		let mut primitives: Vec<PrimitiveEnum<MaterialEnum<TextureEnum>>> = Vec::new();

		let ground = sphere!(0, -1000, 0, 1000, &diffuse!(0.5, 0.5, 0.5, 0.5));

		let sphere_one = sphere!(0, 1, 0, 1, &refract!(&solid_colour!(colour!(1)), 1.5));

		let sphere_two = sphere!(-4, 1, 0, 1, &diffuse!(0.4, 0.2, 0.1, 0.5));

		let sphere_three = sphere!(4, 1, 0, 1, &reflect!(&solid_colour!(0.7, 0.6, 0.5), 0));

		primitives.push(ground);
		primitives.push(sphere_one);
		primitives.push(sphere_two);
		primitives.push(sphere_three);

		let mut rng = SmallRng::from_rng(thread_rng()).unwrap();

		let seed: String = std::iter::repeat(())
			.map(|()| rng.sample(Alphanumeric))
			.map(char::from)
			.take(32)
			.collect();

		println!("\tseed: {}", seed);
		let mut rng: SmallRng = Seeder::from(seed.clone()).make_rng();

		for a in -11..11 {
			for b in -11..11 {
				let center = position!(
					a as Float + 0.9 * rng.gen::<Float>(),
					0.2,
					b as Float + 0.9 * rng.gen::<Float>()
				);

				if (center - position!(4.0, 0.2, 0.0)).mag() > 0.9 {
					let choose_material: Float = rng.gen();
					let colour =
						colour!(rng.gen::<Float>(), rng.gen::<Float>(), rng.gen::<Float>());

					let sphere;

					if choose_material < 0.8 {
						sphere = sphere!(center, 0.2, &diffuse!(&solid_colour!(colour), 0.5));
					} else if choose_material < 0.95 {
						sphere = sphere!(
							center,
							0.2,
							&reflect!(&solid_colour!(colour), rng.gen::<Float>() / 2.0)
						);
					} else {
						sphere = sphere!(center, 0.2, &refract!(&solid_colour!(colour!(1)), 1.5));
					}
					primitives.push(sphere);
				}
			}
		}

		let camera = camera!(
			position!(13, 2, -3),
			position!(0, 0, 0),
			position!(0, 1, 0),
			29,
			16.0 / 9.0,
			0.1,
			10
		);

		let bvh = bvh!(primitives, SplitType::Sah);

		let scene = scene!(camera, sky!(), random_sampler!(), bvh);

		let bvh = scene.acceleration_structure;

		for node in &bvh.nodes {
			for i in node.primitive_offset..(node.primitive_offset + node.number_primitives) {
				let aabb = bvh.primitives[i].get_aabb().unwrap();
				assert!(
					(node.bounds.max - aabb.max).component_min() >= 0.0
						&& (aabb.min - node.bounds.min).component_min() >= 0.0
				);
			}
		}
	}
}
