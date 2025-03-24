
use core::fmt;

use image::Rgb;
// note: handling alpha values will be delegated to when
//      octree is quantizing the image, i'll copy and paste from original image to quantized image.

pub fn get_color_index(color: Rgb<u8>, level: usize) -> usize {
    let mut index: usize = 0;
    let mask = 0b10000000 >> level;
    if color.0[0] & mask != 0 { index |= 0b100; }
    if color.0[1] & mask != 0 { index |= 0b010; }
    if color.0[2] & mask != 0 { index |= 0b001; }

    return index;
}
pub fn add_colors_mut(first_color: &mut Rgb<u32>, second_color: Rgb<u8>) {
    for (a, b) in first_color.0.iter_mut().zip(second_color.0.iter()) {
        *a += u32::from(*b)
    } 
}

pub struct FlatOctreeNode {
    color: Rgb<u32>,
    pixel_count: u32,
    children: [Option<usize>; 8],
}
pub struct FlatOctree {
    root: usize,
    nodes: Vec<FlatOctreeNode>,
}

impl FlatOctreeNode {
    pub fn new() -> Self {
        Self {
            color: Rgb([0, 0, 0]),
            pixel_count: 0,
            children: [None; 8],
        }
    }
}
impl fmt::Display for FlatOctreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [r, g, b] = self.color.0.map(|x| x as u8);
        let children: Vec<String> = self.children.iter()
            .filter_map(|c| c.as_ref().map(|x| format!("{}", x)))
            .collect();
        write!(f, "<color: #{:02X}{:02X}{:02X}, pixel_count: {}, children: [{}]>", r, g, b, self.pixel_count, children.join(", "))
    }
}

impl FlatOctree {
    pub fn new() -> Self {
        let node = FlatOctreeNode::new();
        Self { 
            root: 0,
            nodes: vec![node], 
        }
    }

    pub fn add_color(&mut self, color: Rgb<u8>) {
        let mut current_node_index: usize = self.root;
        for level_index in 0..=7 {
            let nodes_len = self.nodes.len();
            let nodes = &mut self.nodes;
            let node = nodes.get_mut(current_node_index);
            let child_index = get_color_index(color, level_index);
            if let Some(node) = node {
                if level_index >= 7 {
                    add_colors_mut(&mut node.color, color);
                    node.pixel_count += 1;
                }
                let mut node_index = node.children.get(child_index).and_then(|x| *x);
                if node_index.is_none() {
                    let mut new_node = FlatOctreeNode::new();
                    new_node.children[child_index] = Some(nodes_len);
                    nodes.push(new_node);
                    node_index.replace(nodes_len);
                }
                current_node_index = node_index.unwrap();

            }
        }

    }
}

impl fmt::Display for FlatOctree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // i will implement this in bevy soon.
        let node = self.nodes.iter().map(|c| format!("{}", c)).collect::<Vec<String>>();
        write!(f, "{:?}", node)
    }
}
