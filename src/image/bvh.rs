use crate::image::aabb::AABB;
use crate::image::hittables::Axis;
use crate::image::ray::Ray;
use crate::image::scene::HittablesType;

use crate::image::tracing::HittableTrait;

type NodeIndex = u32;

pub struct BVH {
    root_nodes: Vec<NodeIndex>,
    nodes: Vec<Node>,
}

impl BVH {
    pub fn new(hittables: &HittablesType) -> Self {
        let mut new_bvh = Self {
            root_nodes: Vec::new(),
            nodes: Vec::new(),
        };
        new_bvh.generate_bvh(hittables);
        new_bvh
    }

    fn generate_bvh(&mut self, hittables: &HittablesType) {
        self.nodes = Vec::new();

        // get hittable indexes
        let mut index_vec = Vec::new();
        for i in 0..hittables.len() {
            index_vec.push(i as u32);
        }

        // pack index and AABB min value into typle
        let mut sorted_tuples = Vec::new();
        for (hittable, index) in hittables.iter().zip(index_vec) {
            sorted_tuples.push((hittable.get_aabb().unwrap(), index));
        }

        // returns ceil(vec_size / 2) unless size is 1 which then it returns 1
        let half_size = std::cmp::max((hittables.len() as f64 / 2.0).ceil() as usize, 1);

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

    fn new_node(&mut self, hittable_tuples: &mut [(AABB, u32)]) -> NodeIndex {
        // get node AABB that contains all AABB's in hittable_tuples
        let containing_aabb =
            AABB::new_contains(&hittable_tuples.iter().map(|(aabb, _)| *aabb).collect());

        let mut new_node = Node::new(
            containing_aabb,
            hittable_tuples.iter().map(|(_, index)| *index).collect(),
        );

        if hittable_tuples.len() != 1 {
            // returns ceil(vec_size / 2) since size != 1
            let half_size = (hittable_tuples.len() as f64 / 2.0).ceil() as usize;

            // random sorting axis
            let axis = Axis::random_axis();

            // sort min values in axis
            hittable_tuples.sort_by(|a, b| {
                axis.get_axis_value(a.0.min)
                    .partial_cmp(&axis.get_axis_value(b.0.min))
                    .unwrap()
            });

            // create and add child nodes
            let mut chunks: Vec<&mut [(AABB, u32)]> =
                hittable_tuples.chunks_mut(half_size).collect();
            let left_index = self.new_node(chunks[0]);
            let right_index = self.new_node(chunks[1]);
            new_node.add_child_nodes(left_index, right_index);
        }
        self.nodes.push(new_node);
        self.nodes.len() as u32 - 1
    }

    pub fn get_intersection_candidates(&self, ray: &Ray) -> Vec<NodeIndex> {
        let mut hittable_indices = Vec::new();
        for root_node in &self.root_nodes {
            hittable_indices.extend(self.get_indices(*root_node, ray));
        }
        hittable_indices
    }

    fn get_indices(&self, node: NodeIndex, ray: &Ray) -> Vec<NodeIndex> {
        let node = &self.nodes[node as usize];

        if !node.aabb.does_int(ray) {
            return Vec::new();
        }

        match (node.left_child, node.right_child) {
            (Some(left), Some(right)) => {
                let mut a = self.get_indices(left, ray);
                a.extend(self.get_indices(right, ray));
                a
            }
            (Some(left), None) => self.get_indices(left, ray),
            (None, Some(right)) => self.get_indices(right, ray),
            (None, None) => node.data.clone(),
        }
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
