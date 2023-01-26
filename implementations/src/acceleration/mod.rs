use crate::{
	aabb::{AABound, AABB},
	acceleration::split::{Split, SplitType},
	rt_core::*,
	utility::sort_by_indices,
	Axis,
};
use std::{collections::VecDeque, marker::PhantomData};

#[cfg(all(feature = "f64"))]
use std::f64::EPSILON;

#[cfg(not(feature = "f64"))]
use std::f32::EPSILON;

pub mod aabb;
pub mod split;

#[derive(Debug, Clone, Copy)]
pub struct PrimitiveInfo {
	pub index: usize,
	pub min: Vec3,
	pub max: Vec3,
	pub center: Vec3,
}

impl PrimitiveInfo {
	fn new<P: Primitive + AABound, M: Scatter>(index: usize, primitive: &P) -> PrimitiveInfo {
		let aabb = primitive.get_aabb();
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

pub struct Bvh<P: Primitive, M: Scatter> {
	split_type: SplitType,
	nodes: Vec<Node>,
	pub primitives: Vec<P>,
	pub lights: Vec<usize>,
	phantom: PhantomData<M>,
}

impl<P, M> Bvh<P, M>
where
	P: Primitive + AABound,
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
			.map(|(index, primitive)| PrimitiveInfo::new::<P, M>(index, primitive))
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
	pub fn number_nodes(&self) -> usize {
		self.nodes.len()
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
			AABB::merge(&mut bounds, AABB::new(info.min, info.max));
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
				AABB::extend_contains(&mut center_bounds, info.center);
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
}

impl<P, M> AccelerationStructure for Bvh<P, M>
where
	P: Primitive<Material = M>,
	M: Scatter,
{
	type Object = P;
	type Material = M;
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

		let offset_lens = self.get_intersection_candidates(ray);

		let intersection = object.get_int(ray);

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
				if let Some(current_hit) = tobject.get_int(ray) {
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
	fn get_samplable(&self) -> &[usize] {
		&self.lights
	}
	fn get_object(&self, index: usize) -> Option<&P> {
		self.primitives.get(index)
	}
}

#[derive(Debug)]
pub struct Node {
	bounds: AABB,
	children: Option<[usize; 2]>,
	primitive_offset: usize,
	number_primitives: usize,
}

impl Node {
	fn new(bounds: AABB, primitive_offset: usize, number_primitives: usize) -> Self {
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
