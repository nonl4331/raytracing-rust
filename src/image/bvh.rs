use crate::image::hittables::Axis;
use crate::image::scene::HittablesType;

use crate::image::tracing::HittableTrait;
use ultraviolet::DVec3;

type NodeIndex = u32;

pub struct BVH {
    nodes: Vec<Node>,
}

impl BVH {
    pub fn new(hittables: &HittablesType) -> Self {
        let mut new_bvh = Self { nodes: Vec::new() };
        new_bvh.generate_bvh(hittables);
        new_bvh
    }

    fn generate_bvh(&mut self, hittables: &HittablesType) {
        self.nodes = Vec::new();

        // get a readcopy of scene hittables
        let hittables = hittables.read().unwrap();

        // get hittable indexes
        let mut index_vec = Vec::new();
        for i in 0..hittables.len() {
            index_vec.push(i as u32);
        }

        // pack index and AABB min value into typle
        let mut sorted_tuples = Vec::new();
        for (hittable, index) in hittables.iter().zip(index_vec) {
            sorted_tuples.push((hittable.get_aabb().unwrap().min, index));
        }

        // returns ceil(vec_size / 2) unless size is 1 which then it returns 1
        let half_size = std::cmp::max((hittables.len() as f64 / 2.0).ceil() as usize, 1);

        // get random axis to sort along
        let axis = Axis::random_axis();

        // sort min values in axis
        sorted_tuples.sort_by(|a, b| {
            axis.get_axis_value(a.0)
                .partial_cmp(&axis.get_axis_value(b.0))
                .unwrap()
        });

        // create a node with each half
        for half_split in sorted_tuples.chunks_mut(half_size) {
            self.new_node(half_split);
        }
    }

    fn new_node(&mut self, hittable_tuples: &mut [(DVec3, u32)]) -> NodeIndex {
        let node_index = self.nodes.len();
        let mut new_node;

        if hittable_tuples.len() == 1 {
            new_node = Node::new(Some(hittable_tuples[0].1));
        } else {
            // create node with no data since has children

            new_node = Node::new(None);

            // returns ceil(vec_size / 2) since size != 1
            let half_size = (hittable_tuples.len() as f64 / 2.0).ceil() as usize;

            // random sorting axis
            let axis = Axis::random_axis();

            // sort min values in axis
            hittable_tuples.sort_by(|a, b| {
                axis.get_axis_value(a.0)
                    .partial_cmp(&axis.get_axis_value(b.0))
                    .unwrap()
            });

            // create and add child nodes
            let mut chunks: Vec<&mut [(DVec3, u32)]> =
                hittable_tuples.chunks_mut(half_size).collect();
            let left_index = self.new_node(chunks[0]);
            let right_index = self.new_node(chunks[1]);
            new_node.add_child_nodes(left_index, right_index);
        }
        self.nodes.push(new_node);
        node_index as u32
    }
}

struct Node {
    left_child: Option<NodeIndex>,
    right_child: Option<NodeIndex>,

    data: Option<u32>,
}

impl Node {
    pub fn new(data: Option<u32>) -> Self {
        Node {
            left_child: None,
            right_child: None,
            data,
        }
    }
    pub fn add_child_nodes(&mut self, left_index: NodeIndex, right_index: NodeIndex) {
        self.left_child = Some(left_index);
        self.right_child = Some(right_index);
    }
}
