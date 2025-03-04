
use core::fmt;

use image::Rgb;

use super::octree_flat::{add_colors_mut, get_color_index};

const MAX_DEPTH: u8 = 8;
pub struct OctreeNode {
    color: Rgb<u32>,
    pixel_count: u32,
    pub children: [Option<Box<OctreeNode>>; 8],
}
pub struct Octree {
    pub root: OctreeNode,
}

impl Octree {
    pub fn new() -> Self {
        Self { root: OctreeNode::new() }
    }
    pub fn add_color(&mut self, color: Rgb<u8>) {
        self.root.add_color(color, 0);
    }
}
impl fmt::Display for Octree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // i will implement this in bevy soon.
        let node = &self.root;
        write!(f, "{}", node.to_string())
    }
}

impl OctreeNode {
    pub fn new() -> Self {
        Self {
            color: Rgb([0, 0, 0]),
            pixel_count: 0,
            children: std::array::from_fn(|_| None),
        }
    }
    pub fn add_color(&mut self, color: Rgb<u8>, level: usize) {
        if level >= usize::from(MAX_DEPTH) {
            add_colors_mut(&mut self.color, color);
            self.pixel_count += 1;
            return;
        }
        let index = get_color_index(color, level);
        let child = &mut self.children[index];
        if child.is_none() {
            self.children[index] = Some(Box::new(OctreeNode::new()));
        }
        self.children[index].as_mut().unwrap().add_color(color, level + 1);
    }
}

impl fmt::Display for OctreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [r, g, b] = self.color.0.map(|x| x as u8);
        let children: Vec<String> = self.children.iter()
            .filter_map(|c| c.as_ref().map(|x| format!("{}", x)))
            .collect();
        write!(f, "<color: #{:02X}{:02X}{:02X}, pixel_count: {}, children: [{}]>", r, g, b, self.pixel_count, children.join(", "))
    }
}
