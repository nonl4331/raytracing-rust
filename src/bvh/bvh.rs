use crate::bvh::aabb::AABB;
use crate::bvh::split::{Split, SplitType};
use crate::ray_tracing::ray::Ray;
use crate::ray_tracing::tracing::Primitive;

use ultraviolet::Vec3;

use crate::ray_tracing::{primitives::Axis, tracing::PrimitiveTrait};

use std::collections::VecDeque;

#[derive(Debug)]
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
    nodes: Vec<NewNode>,
}

impl BVH {
    pub fn new(primitives: &mut Vec<Primitive>, split_type: SplitType) -> Self {
        let mut new_bvh = Self {
            split_type,
            nodes: Vec::new(),
        };
        let mut primitives_info: Vec<PrimitiveInfo> = primitives
            .iter()
            .enumerate()
            .map(|(index, primitive)| PrimitiveInfo::new(index, primitive))
            .collect();

        let primitives_info = new_bvh.build_bvh(&mut primitives_info);

        *primitives = primitives_info
            .iter()
            .map(|&index| std::mem::replace(&mut primitives[index], Primitive::None))
            .collect();
        new_bvh
    }

    fn build_bvh(&mut self, primitives_info: &mut Vec<PrimitiveInfo>) -> Vec<usize> {
        self.nodes = Vec::new();
        let end = primitives_info.len();
        if end == 0 {
            return Vec::new();
        }

        let mut left_queue: VecDeque<(usize, usize)> = VecDeque::new();
        let mut right_queue: VecDeque<(usize, usize)> = VecDeque::new();
        let mut parent_queue: VecDeque<usize> = VecDeque::new();
        let mut ordered_primitives = Vec::new();

        let start: usize = 0;
        left_queue.push_back((start, end));
        while left_queue.len() > 0 || right_queue.len() > 0 {
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
                let parent_index = parent_queue[0];
                let node_len = self.nodes.len();
                match is_left {
                    true => {
                        self.nodes[parent_index].set_child(node_len, 0);
                    }
                    false => {
                        self.nodes[parent_index].set_child(node_len, 1);
                        parent_queue.pop_front().unwrap();
                    }
                }
            }

            let mut bounds = None;
            for info in primitives_info[start..end].iter() {
                AABB::merge(&mut bounds, AABB::new(info.min, info.max));
            }

            let number_primitives = end - start;

            if number_primitives == 1 {
                for i in start..end {
                    let primitive_number = primitives_info[i].index;
                    ordered_primitives.push(primitive_number);
                }
            } else {
                let mut center_bounds = None;
                for info in primitives_info[start..end].iter() {
                    AABB::extend_contains(&mut center_bounds, info.center);
                }

                let center_bounds = center_bounds.unwrap();

                axis = Axis::get_max_axis(&center_bounds.get_extent());

                if axis.get_axis_value(center_bounds.min) == axis.get_axis_value(center_bounds.max)
                {
                    for i in start..end {
                        let primitive_number = primitives_info[i].index;
                        ordered_primitives.push(primitive_number);
                    }
                } else {
                    parent_queue.push_front(self.nodes.len());

                    let mid =
                        self.split_type
                            .split(start, end, &center_bounds, &axis, primitives_info);
                    left_queue.push_back((start, mid));
                    right_queue.push_back((mid, end));
                }
            }

            self.nodes.push(NewNode::new(
                axis,
                bounds.unwrap(),
                ordered_primitives.len(),
                end - start,
            ));
        }
        ordered_primitives
    }
    pub fn get_intersection_candidates(&self, ray: &Ray) -> (usize, usize) {
        let mut offset = 0;
        let mut len = 0;

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
                    if children[0] == 100000000 || children[1] == 100000000 {}
                    node_stack.push_back(children[0]);
                    node_stack.push_back(children[1]);
                }
                None => {
                    offset = offset.min(node.primitive_offset);
                    len += node.number_primitives;
                }
            }
        }
        (offset, len)
    }
}

#[derive(Debug)]
pub struct NewNode {
    bounds: AABB,
    children: Option<[usize; 2]>,
    split_axis: Axis,
    primitive_offset: usize,
    number_primitives: usize,
}

impl NewNode {
    fn new(
        split_axis: Axis,
        bounds: AABB,
        primitive_offset: usize,
        number_primitives: usize,
    ) -> Self {
        NewNode {
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
                let mut children = [100000000, 100000000];
                children[index] = child_index;
                self.children = Some(children);
            }
        }
    }
}
