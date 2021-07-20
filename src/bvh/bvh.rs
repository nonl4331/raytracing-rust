use crate::bvh::aabb::AABB;
use crate::ray_tracing::tracing::Primitive;
use std::sync::Arc;
use ultraviolet::Vec3;

use crate::image::scene::PrimitivesType;

use crate::ray_tracing::{primitives::Axis, ray::Ray, tracing::PrimitiveTrait};

use std::collections::VecDeque;

const MAX_IN_NODE: u32 = 128;

type NodeIndex = u32;

pub enum SplitType {
    SAH,
    HLBVH,
    Middle,
    EqualCounts,
}

struct PrimitiveInfo {
    index: usize,
    min: Vec3,
    max: Vec3,
    center: Vec3,
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
    root_nodes: Vec<NodeIndex>,
    nodes: Vec<NewNode>,
}

impl BVH {
    pub fn new(primitives: &PrimitivesType, split_type: SplitType) -> Self {
        let mut new_bvh = Self {
            split_type,
            root_nodes: Vec::new(),
            nodes: Vec::new(),
        };
        let primitives_info: Vec<PrimitiveInfo> = primitives
            .iter()
            .enumerate()
            .map(|(index, primitive)| PrimitiveInfo::new(index, primitive))
            .collect();

        new_bvh.build_bvh(&primitives_info);
        new_bvh
    }

    fn build_bvh(&mut self, primitives_info: &Vec<PrimitiveInfo>) -> Vec<usize> {
        self.nodes = Vec::new();
        let mut end = primitives_info.len();
        if end == 0 {
            return Vec::new();
        }

        let mut left_queue: VecDeque<(usize, usize)> = VecDeque::new();
        let mut right_queue: VecDeque<(usize, usize)> = VecDeque::new();
        let mut parent_queue: VecDeque<usize> = VecDeque::new();
        let mut ordered_primitives = Vec::new();

        let mut start: usize = 0;
        let mut total_nodes = 0;
        left_queue.push_back((start, end));
        let mut is_left: bool = true;
        let mut axis = Axis::X;

        while left_queue.len() > 0 || right_queue.len() > 0 {
            let (start, end) = if left_queue.len() > 0 {
                is_left = true;
                left_queue.pop_front().unwrap()
            } else {
                is_left = false;

                right_queue.pop_front().unwrap()
            };

            let mut bounds = None;
            for info in primitives_info {
                AABB::merge(&mut bounds, AABB::new(info.min, info.max));
            }

            let number_primitives = start - end;

            // can't split a single primitive
            if number_primitives == 1 {
                let first_primitive_offset = ordered_primitives.len();
                for i in start..end {
                    let primitive_number = primitives_info[i].index;
                    ordered_primitives.push(primitive_number);
                }
            } else {
                // calculate bounds choose split
                let mut center_bounds = None;

                for info in primitives_info {
                    AABB::extend_contains(&mut center_bounds, info.center);
                }
                let center_bounds = center_bounds.unwrap();
                let axis = Axis::get_max_axis(&center_bounds.get_extent());
                // partition into two sets and build children

                let mid = (start + end) / 2;

                // don't split primitives when center of AABB overlaps
                if axis.get_axis_value(center_bounds.min) == axis.get_axis_value(center_bounds.max)
                {
                    let first_primitive_offset = ordered_primitives.len();
                    for i in start..end {
                        let primitive_number = primitives_info[i].index;
                        ordered_primitives.push(primitive_number);
                    }

                // regular spliting
                } else {
                    parent_queue.push_back(self.nodes.len());

                    // let (left_node, right_node) = split_node();
                    left_queue.push_back((start, mid));
                    right_queue.push_back((mid, end));
                }
            }

            self.nodes.push(NewNode::new(
                axis,
                bounds.unwrap(),
                ordered_primitives.len(),
                start - end,
            ));

            if parent_queue.len() != 0 {
                let parent_index = parent_queue[0];
                match is_left {
                    true => {
                        self.nodes[parent_index].set_child(self.nodes.len(), 0);
                    }
                    false => {
                        self.nodes[parent_index].set_child(self.nodes.len(), 1);
                    }
                }
            }

            if is_left {
                left_queue.pop_front().unwrap();
            } else {
                parent_queue.pop_front().unwrap();
                right_queue.pop_front().unwrap();
            }
            if left_queue.len() == 0 {
                // add data to ordered data
                for i in start..end {
                    ordered_primitives.push(primitives_info[i].index);
                }
            }
            total_nodes += 1;
        }
        ordered_primitives
    }
}

pub struct Node {
    left_child: Option<NodeIndex>,
    right_child: Option<NodeIndex>,

    //axis: Axis,
    aabb: AABB,

    data: Vec<u32>,
}

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
            Some(children) => {
                children[index] = child_index;
            }
            None => {
                let mut children = [0, 0];
                children[index] = child_index;
                self.children = Some(children);
            }
        }
    }
    fn new_from_children(split_axis: Axis, children: ([usize; 2], &[NewNode; 2])) -> Self {
        let bounds = AABB::new_contains(&children.1.iter().map(|child| child.bounds).collect());
        let primitive_offset = children.1[0].primitive_offset;
        let number_primitives = children.1.iter().map(|child| child.number_primitives).sum();
        let children = Some(children.0);

        NewNode {
            bounds,
            children,
            split_axis,
            primitive_offset,
            number_primitives,
        }
    }
}

impl Node {
    pub fn new(aabb: AABB, data: Vec<u32>) -> Self {
        Node {
            left_child: None,
            right_child: None,
            aabb,
            data,
        }
    }

    pub fn add_child_nodes(&mut self, left_index: NodeIndex, right_index: NodeIndex) {
        self.left_child = Some(left_index);
        self.right_child = Some(right_index);
    }
}
