use crate::bvh::aabb::AABB;

use crate::image::scene::PrimitivesType;

use crate::ray_tracing::{primitives::Axis, ray::Ray, tracing::PrimitiveTrait};

type NodeIndex = u32;

pub struct BVH {
    root_nodes: Vec<NodeIndex>,
    nodes: Vec<Node>,
}

impl BVH {
    pub fn new(primitives: &PrimitivesType) -> Self {
        let mut new_bvh = Self {
            root_nodes: Vec::new(),
            nodes: Vec::new(),
        };
        new_bvh.generate_bvh(primitives);
        new_bvh
    }

    fn generate_bvh(&mut self, primitives: &PrimitivesType) {
        self.nodes = Vec::new();

        // get primitive indexes
        let mut index_vec = Vec::new();
        for i in 0..primitives.len() {
            index_vec.push(i as u32);
        }

        // pack index and AABB min value into typle
        let mut sorted_tuples = Vec::new();
        for (primitive, index) in primitives.iter().zip(index_vec) {
            sorted_tuples.push((primitive.get_aabb().unwrap(), index));
        }

        // returns ceil(vec_size / 2) unless size is 1 which then it returns 1
        let half_size = std::cmp::max((primitives.len() as f32 / 2.0).ceil() as usize, 1);

        // get random axis to sort along
        let axis = Axis::random_axis();

        // sort min values in axis
        sorted_tuples.sort_by(|a, b| {
            axis.get_axis_value(a.0.min)
                .partial_cmp(&axis.get_axis_value(b.0.min))
                .unwrap()
        });

        // create a node with each half
        for half_split in sorted_tuples.chunks_mut(half_size) {
            let new_node = self.new_node(half_split);
            self.root_nodes.push(new_node);
        }
    }

    fn new_node(&mut self, primitive_tuples: &mut [(AABB, u32)]) -> NodeIndex {
        // get node AABB that contains all AABB's in primitive_tuples
        let containing_aabb =
            AABB::new_contains(&primitive_tuples.iter().map(|(aabb, _)| *aabb).collect());

        let mut new_node = Node::new(
            containing_aabb,
            primitive_tuples.iter().map(|(_, index)| *index).collect(),
        );

        if primitive_tuples.len() != 1 {
            // returns ceil(vec_size / 2) since size != 1
            let half_size = (primitive_tuples.len() as f32 / 2.0).ceil() as usize;

            // random sorting axis
            let axis = Axis::random_axis();

            // sort min values in axis
            primitive_tuples.sort_by(|a, b| {
                axis.get_axis_value(a.0.min)
                    .partial_cmp(&axis.get_axis_value(b.0.min))
                    .unwrap()
            });

            // create and add child nodes
            let mut chunks: Vec<&mut [(AABB, u32)]> =
                primitive_tuples.chunks_mut(half_size).collect();
            let left_index = self.new_node(chunks[0]);
            let right_index = self.new_node(chunks[1]);
            new_node.add_child_nodes(left_index, right_index);
        }
        self.nodes.push(new_node);
        self.nodes.len() as u32 - 1
    }

    pub fn get_intersection_candidates(&self, ray: &Ray) -> Vec<NodeIndex> {
        let mut primitive_indices = Vec::new();
        for &root_node in &self.root_nodes {
            primitive_indices.extend(self.get_indices(root_node, ray));
        }
        primitive_indices
    }

    fn get_indices(&self, node: NodeIndex, ray: &Ray) -> Vec<NodeIndex> {
        let mut queue: std::collections::VecDeque<NodeIndex> = std::collections::VecDeque::new();
        let mut result: Vec<NodeIndex> = Vec::new();

        queue.push_back(node);

        while queue.len() > 0 {
            let idx = queue.pop_front().unwrap();
            let node = &self.nodes[idx as usize];

            if !node.aabb.does_int(ray) {
                continue;
            }

            match (node.left_child, node.right_child) {
                (Some(left), Some(right)) => {
                    queue.push_back(left);
                    queue.push_back(right);
                }
                (Some(left), None) => {
                    queue.push_back(left);
                }
                (None, Some(right)) => {
                    queue.push_back(right);
                }
                (None, None) => {
                    result.extend(node.data.clone());
                }
            }
        }

        result
    }
}

struct Node {
    left_child: Option<NodeIndex>,
    right_child: Option<NodeIndex>,

    aabb: AABB,

    data: Vec<u32>,
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
