use crate::bvh::{
    aabb::AABB,
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
    index: usize,
    min: Vec3,
    max: Vec3,
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

pub struct BVH {
    split_type: SplitType,
    nodes: Vec<Node>,
}

impl BVH {
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

        let primitives_info = bvh.build_bvh(&mut primitives_info);

        *primitives = primitives_info
            .iter()
            .map(|&index| std::mem::replace(&mut primitives[index], Primitive::None))
            .collect();

        bvh
    }

    fn build_bvh(&mut self, primitives_info: &mut Vec<PrimitiveInfo>) -> Vec<usize> {
        self.nodes = Vec::new();
        let end = primitives_info.len();
        if end == 0 {
            return Vec::new();
        }

        let mut left_queue: VecDeque<(usize, usize)> = VecDeque::new();
        let mut right_queue: VecDeque<(usize, usize)> = VecDeque::new();
        let mut parent_queue: VecDeque<(usize, Vec<PrimitiveInfo>)> = VecDeque::new();
        let mut ordered_primitives = Vec::new();

        let start: usize = 0;
        left_queue.push_back((start, end));
        let mut pop_parent = false;
        while left_queue.len() > 0 || right_queue.len() > 0 {
            let mut current_primitives_info;
            let mut axis = Axis::X;
            let is_left;

            let (start, end) = if left_queue.len() > 0 {
                is_left = true;
                left_queue.pop_front().unwrap()
            } else {
                is_left = false;
                right_queue.pop_front().unwrap()
            };

            if parent_queue.len() != 0 {
                let parent_index = parent_queue[0].0;
                current_primitives_info = parent_queue[0].1.clone();
                let node_len = self.nodes.len();
                match is_left {
                    true => {
                        self.nodes[parent_index].set_child(node_len, 0);
                    }
                    false => {
                        self.nodes[parent_index].set_child(node_len, 1);
                        pop_parent = true;
                    }
                }
            } else {
                current_primitives_info = primitives_info.to_vec();
            }

            let mut bounds = None;
            for info in current_primitives_info[start..end].iter() {
                AABB::merge(&mut bounds, AABB::new(info.min, info.max));
            }

            let number_primitives = end - start;

            if number_primitives == 1 {
                for i in start..end {
                    ordered_primitives.push(current_primitives_info[i].index);
                }

                if pop_parent == true {
                    parent_queue.pop_front().unwrap();
                    pop_parent = false;
                }
            } else {
                let mut center_bounds = None;
                for info in current_primitives_info[start..end].iter() {
                    AABB::extend_contains(&mut center_bounds, info.center);
                }

                let center_bounds = center_bounds.unwrap();

                axis = Axis::get_max_axis(&center_bounds.get_extent());

                if axis.get_axis_value(center_bounds.min) == axis.get_axis_value(center_bounds.max)
                {
                    for i in start..end {
                        ordered_primitives.push(current_primitives_info[i].index);
                    }

                    if pop_parent == true {
                        parent_queue.pop_front().unwrap();
                        pop_parent = false;
                    }
                } else {
                    let mid = self.split_type.split(
                        start,
                        end,
                        &center_bounds,
                        &axis,
                        &mut current_primitives_info,
                    );

                    parent_queue.push_front((self.nodes.len(), current_primitives_info.to_vec()));

                    if pop_parent == true {
                        parent_queue.remove(1);
                        pop_parent = false;
                    }

                    left_queue.push_front((start, mid));
                    right_queue.push_front((mid, end));
                }
            }
            let mut len = ordered_primitives.len();
            if number_primitives == 1 {
                len -= 1;
            }

            self.nodes
                .push(Node::new(axis, bounds.unwrap(), len, end - start));
        }
        ordered_primitives
    }
    pub fn get_intersection_candidates(&self, ray: &Ray) -> Vec<(usize, usize)> {
        let mut offset_len = Vec::new();

        let mut node_stack = VecDeque::new();
        node_stack.push_back(0);
        while node_stack.len() > 0 {
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

#[derive(Debug)]
pub struct Node {
    bounds: AABB,
    children: Option<[usize; 2]>,
    split_axis: Axis,
    primitive_offset: usize,
    number_primitives: usize,
}

impl Node {
    fn new(
        split_axis: Axis,
        bounds: AABB,
        primitive_offset: usize,
        number_primitives: usize,
    ) -> Self {
        Node {
            bounds,
            children: None,
            split_axis,
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
    use super::*;
    use crate::ray_tracing::material::Material;
    use crate::ray_tracing::material::Refract;
    use crate::ray_tracing::primitives::Sphere;
    use crate::ray_tracing::ray::Colour;

    #[test]
    fn primitive_info_new() {
        let sphere = Primitive::Sphere(Sphere::new(
            Vec3::one(),
            0.2,
            Material::Refract(Refract::new(Colour::one(), 1.5)),
        ));
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
