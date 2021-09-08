use crate::acceleration::{
    aabb::Aabb,
    split::{Split, SplitType},
};

use crate::ray_tracing::{
    primitives::{Axis, Primitive},
    ray::Ray,
    tracing::PrimitiveTrait,
};

use std::collections::VecDeque;

use ultraviolet::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct PrimitiveInfo {
    pub index: usize,
    pub min: Vec3,
    pub max: Vec3,
    pub center: Vec3,
}

impl PrimitiveInfo {
    fn new(index: usize, primitive: &Primitive) -> PrimitiveInfo {
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

pub struct Bvh {
    split_type: SplitType,
    nodes: Vec<Node>,
}

impl Bvh {
    pub fn new(primitives: &mut Vec<Primitive>, split_type: SplitType) -> Self {
        let mut bvh = Self {
            split_type,
            nodes: Vec::new(),
        };
        let mut primitives_info: Vec<PrimitiveInfo> = primitives
            .iter()
            .enumerate()
            .map(|(index, primitive)| PrimitiveInfo::new(index, primitive))
            .collect();

        bvh.build_bvh(&mut Vec::new(), 0, &mut primitives_info);

        *primitives = primitives_info
            .iter()
            .map(|&info| std::mem::replace(&mut primitives[info.index], Primitive::None))
            .collect();
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
                .abs()
                < 100.0 * f32::EPSILON
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

    use crate::*;

    use super::*;

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
        let scene = crate::image::generate::scene_one(SplitType::EqualCounts, 16.0 / 9.0);
        let bvh = scene.bvh;

        for node in &bvh.nodes {
            for i in node.primitive_offset..(node.primitive_offset + node.number_primitives) {
                let aabb = scene.primitives[i].get_aabb().unwrap();
                assert!(
                    (node.bounds.max - aabb.max).component_min() >= 0.0
                        && (aabb.min - node.bounds.min).component_min() >= 0.0
                );
            }
        }
    }
}
