
use image::Rgb;
// note: handling alpha values will be delegated to when
//      octree is quantizing the image, i'll copy and paste from original image to quantized image.

fn get_color_index(color: Rgb<u8>, level: usize) -> usize {
    let mut index: usize = 0;
    let mask = 0b10000000 >> level;
    if color.0[0] & mask != 0 { index |= 0b100 }
    if color.0[1] & mask != 0 { index |= 0b010 }
    if color.0[2] & mask != 0 { index |= 0b001 }

    return index;
}
fn add_colors_mut(first_color: &mut Rgb<u32>, second_color: Rgb<u8>) {
    for (a, b) in first_color.0.iter_mut().zip(second_color.0.iter()) {
        *a += u32::from(*b)
    } 
}

pub struct OctreeNode {
    color: Rgb<u32>,
    pixel_count: u32,
    children: [Option<usize>; 8],
}
pub struct Octree {
    root: usize,
    nodes: Vec<OctreeNode>,
}

impl OctreeNode {
    pub fn new() -> Self {
        Self {
            color: Rgb([0, 0, 0]),
            pixel_count: 0,
            children: [None; 8],
        }
    }
}

impl Octree {
    pub fn new() -> Self {
        let node = OctreeNode::new();
        Self { 
            root: 0,
            nodes: vec![node], 
        }
    }

    fn add_color(&mut self, color: Rgb<u8>) {
        let mut current_node_index: usize = 0;
        for level_index in 0..8 {
            let nodes_len = self.nodes.len();
            let nodes = &mut self.nodes;
            let node = nodes.get_mut(current_node_index);
            let child_index = get_color_index(color, level_index);
            if let Some(node) = node {
                if level_index >= 8 {
                    add_colors_mut(&mut node.color, color);
                    node.pixel_count += 1;
                }
                let node_index = node.children.get(child_index).and_then(|x| *x);
                if let Some(node_index) = node_index {
                    current_node_index = node_index;
                } else {
                    let mut new_node = OctreeNode::new();
                    new_node.children[child_index] = Some(nodes_len);
                    nodes.push(new_node);
                }

            }
        }

    }
}

pub fn test() {

}
