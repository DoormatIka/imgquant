
use core::fmt;
use crate::core::rgb_helpers::IRgb;
use std::{cell::RefCell, rc::{Rc, Weak}};
use image::Rgb;
// note: 0, 1, 2 corresponds to R, G, B

pub fn get_color_index(color: IRgb<u8>, level: usize) -> usize {
    let [r, g, b] = color.0;
    let mut index: usize = 0;
    let mask = 0b10000000 >> level;
    if r & mask != 0 { index |= 0b100; }
    if g & mask != 0 { index |= 0b010; }
    if b & mask != 0 { index |= 0b001; }

    return index;
}

#[derive(Clone, Debug)]
pub struct OctreeNode {
    children: [Option<Rc<RefCell<OctreeNode>>>; 8],
    color: IRgb<u32>,
    pixel_count: u32,
    palette_index: u32,
}

type LevelVec = Vec<Vec<Weak<RefCell<OctreeNode>>>>;
pub struct LeafOctree {
    depth: usize,
    root: OctreeNode,
    levels: LevelVec,
}

impl LeafOctree {
    pub fn new(depth: usize) -> Self {
        let mut nodes = Vec::new();
        nodes.reserve(10_000); // over-allocating at higher levels (level 1 to 3~)
        Self {
            root: OctreeNode::new(),
            levels: vec![nodes; depth + 1],
            depth,
        }
    }

    pub fn make_palette(&mut self, color_count: i32) -> Vec<Rgb<u8>> {
        let mut palette = Vec::<Rgb<u8>>::new();
        let mut palette_index = 0;
        let leaves = self.get_leaf_nodes();
        let mut leaf_count = leaves.len() as i32;

        for level_index in (0..(self.depth as usize - 1)).rev() {
            let level = &mut self.levels[level_index];
            for node in level {
                if let Some(node) = node.upgrade() {
                    let mut node = node.borrow_mut();
                    leaf_count -= node.remove_leaves();
                }
                if leaf_count <= color_count {
                    break;
                }
            }
            if leaf_count <= color_count {
                break;
            }
        }

        for ele in self.levels.iter_mut() {
            ele.clear();
        }

        let mut leaves = self.get_leaf_nodes();
        for node in leaves.iter_mut() {
            if palette_index >= color_count {
                break;
            }
            if let Some(node) = node.upgrade() {
                let mut borrowed_node = node.borrow_mut();
                if borrowed_node.is_leaf() {
                    let [r, g, b] = borrowed_node.color.0;
                    let rgb = IRgb::<u8>([
                        (r / borrowed_node.pixel_count) as u8,
                        (g / borrowed_node.pixel_count) as u8,
                        (b / borrowed_node.pixel_count) as u8,
                    ]);
                    palette.push(rgb);
                }
                borrowed_node.palette_index = palette_index as u32;
                palette_index += 1;
            }
        }

        palette
    }

    pub fn add_color(&mut self, color: IRgb<u8>) {
        self.root.add_color(color, 0, &mut self.levels, self.depth);
    }
    /// Returns the palette index for the closest color in the octree to your given color.
    /// 
    /// # Arguments
    /// * `color` - The color value to find the palette index of.
    /// * `force_find_color` - If `true`, the function will force the octree
    /// to find colors. If `false`, it can return a None.
    ///
    /// # Returns
    /// * `Some(index)` if a suitable match is found.
    /// * `None` if no match is found.
    pub fn get_palette_index(&self, color: IRgb<u8>, force_find_color: bool) -> Option<usize> {
        self.root.get_palette_index(color, 0, force_find_color)
    }
    pub fn get_leaf_nodes(&self) -> Vec<Weak<RefCell<OctreeNode>>> {
        let leaves = self.root.get_leaf_nodes();
        return leaves;
    }
}

impl fmt::Display for LeafOctree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // i will implement this in bevy soon.
        let node = &self.root;
        write!(f, "{}", node.to_string())
    }
}

impl OctreeNode {
    pub fn new() -> Self {
        Self {
            color: IRgb::from([0, 0, 0]),
            pixel_count: 0,
            palette_index: 0,
            children: std::array::from_fn(|_| None),
        }
    }
    pub fn add_color(&mut self, color: IRgb<u8>, level: usize, levels: &mut LevelVec, depth: usize) {
        if level >= depth {
            self.color += color;
            add_colors(&mut self.color, &color);
            self.pixel_count += 1;
            return;
        }
        let index = get_color_index(color, level);
        let child = &mut self.children[index];
        if child.is_none() {
            let node = Rc::new(RefCell::new(OctreeNode::new()));
            levels[level].push(Rc::downgrade(&node));
            self.children[index] = Some(node);
        }
        let mut node = self.children[index].as_ref().unwrap().borrow_mut();
        node.add_color(color, level + 1, levels, depth);
    }
    pub fn get_palette_index(&self, color: IRgb<u8>, level: usize, force_find_color: bool) -> Option<usize> {
        if self.is_leaf() {
            return Some(self.palette_index as usize);
        } else {
            let index = get_color_index(color, level);
            match &self.children[index] {
                Some(cell) => {
                    let c = cell.borrow();
                    return c.get_palette_index(color, level + 1, force_find_color);
                },
                None => {
                    if force_find_color {
                        for node in self.children.iter() {
                            if let Some(node) = node {
                                let c = node.borrow();
                                return c.get_palette_index(color, level + 1, force_find_color);
                            }
                        }
                    }
                    None
                },
            }
        }
    }
    pub fn get_leaf_nodes(&self) -> Vec<Weak<RefCell<OctreeNode>>> {
        let mut leaf_nodes = Vec::<Weak<RefCell<OctreeNode>>>::new();
        for child in self.children.iter() {
            if let Some(child) = child {
                let borrowed_child = child.borrow();
                if borrowed_child.is_leaf() {
                    leaf_nodes.push(Rc::downgrade(child)); // reference counted.
                } else {
                    for element in borrowed_child.get_leaf_nodes() {
                        leaf_nodes.push(element);
                    };
                }
            }
        }

        leaf_nodes
    }
    pub fn remove_leaves(&mut self) -> i32 {
        let mut leaves_removed = 0;
    
        for (_i, child) in self.children.iter_mut().enumerate() {
            if let Some(child) = child {
                let borrowed_child = child.borrow();
                if borrowed_child.is_leaf() {
                    leaves_removed += 1;
                }
                self.pixel_count += borrowed_child.pixel_count;
                add_colors(&mut self.color, &borrowed_child.color);
            }
            *child = None;
        }

        leaves_removed - 1
    }

    pub fn is_leaf(&self) -> bool {
        return self.pixel_count > 0;
    }
}

impl fmt::Display for OctreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let [r, g, b] = self.color.0.map(|x| x as u8);
        let children: Vec<String> = self.children.iter()
            .filter_map(|c| c.as_ref().map(|x| format!("{}", x.borrow())))
            .collect();
        write!(f, "<color: #{:02X}{:02X}{:02X}, pixel_count: {}, children: [{}]>", r, g, b, self.pixel_count, children.join(", "))
    }
}



