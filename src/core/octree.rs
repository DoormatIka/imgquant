
use core::{borrow, fmt};
use std::{cell::RefCell, rc::{Rc, Weak}};

use image::Rgb;

use super::octree_flat::{add_colors_mut, get_color_index};

const MAX_DEPTH: u8 = 8;

#[derive(Clone, Debug)]
pub struct OctreeNode {
    children: [Option<Rc<RefCell<OctreeNode>>>; MAX_DEPTH as usize],
    color: Rgb<u32>,
    pixel_count: u32,
    palette_index: u32,
}

type LevelVec = [Vec<Weak<RefCell<OctreeNode>>>; MAX_DEPTH as usize];
pub struct Octree {
    pub root: OctreeNode,
    pub levels: LevelVec,
}

impl Octree {
    pub fn new() -> Self {
        Self {
            root: OctreeNode::new(),
            levels: std::array::from_fn(|_| Vec::new()),
        }
    }
    pub fn make_palette(&mut self, color_count: usize) -> Vec<Rgb<u8>> {
        let mut palette = Vec::<Rgb<u8>>::new();
        let mut palette_index = 0;
        let leaves = self.get_leaf_nodes();
        let mut leaf_count = leaves.len();

        for level in self.levels.iter_mut().rev() {
            for node in level {
                if let Some(node) = node.upgrade() {
                    let mut node = node.borrow_mut();
                    leaf_count -= node.remove_leaves() as usize;
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

        for node in self.get_leaf_nodes().iter_mut() {
            if palette_index >= color_count {
                break;
            }
            if node.is_leaf() { // is this check really needed
                let [r, g, b] = node.color.0;
                let rgb = Rgb::<u8>([
                    (r / node.pixel_count) as u8,
                    (g / node.pixel_count) as u8,
                    (b / node.pixel_count) as u8,
                ]);
                palette.push(rgb);
            }
            node.palette_index = palette_index as u32;
            palette_index += 1;
        }

        palette
    }
    pub fn add_color(&mut self, color: Rgb<u8>) {
        self.root.add_color(color, 0, &mut self.levels);
    }
    pub fn get_palette_index(&self, color: Rgb<u8>) -> usize {
        self.root.get_palette_index(color, 0)
    }
    pub fn get_leaf_nodes(&self) -> Vec<OctreeNode> {
        let leaves = self.root.get_leaf_nodes();
        let nodes: Vec<OctreeNode> = leaves.iter().map(|c| c.borrow().clone()).collect();
        return nodes;
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
            palette_index: 0,
            children: std::array::from_fn(|_| None),
        }
    }
    pub fn add_color(&mut self, color: Rgb<u8>, level: usize, levels: &mut LevelVec) {
        if level >= usize::from(MAX_DEPTH) {
            add_colors_mut(&mut self.color, color);
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
        node.add_color(color, level + 1, levels);
    }
    pub fn get_palette_index(&self, color: Rgb<u8>, level: usize) -> usize {
        if self.is_leaf() {
            return self.palette_index as usize;
        } else {
            let index = get_color_index(color, level);
            match &self.children[index] {
                Some(cell) => {
                    let c = cell.borrow();
                    return c.get_palette_index(color, level + 1);
                },
                None => {
                    for node in self.children.iter() {
                        if let Some(node) = node {
                            let c = node.borrow();
                            return c.get_palette_index(color, level + 1);
                        }
                    }
                    return 0;
                },
            }
        }
    }
    pub fn get_leaf_nodes(&self) -> Vec<Rc<RefCell<OctreeNode>>> {
        let mut leaf_nodes = Vec::<Rc<RefCell<OctreeNode>>>::new();
        for child in self.children.iter() {
            if let Some(child) = child {
                let borrowed_child = child.borrow();
                if borrowed_child.is_leaf() {
                    leaf_nodes.push(Rc::clone(child)); // reference counted.
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
        let mut result = 0;
        for child in self.children.iter_mut() {
            if let Some(child) = child {
                let borrowed_child = child.borrow_mut();
                for (own_color, child_color) in self.color.0.iter_mut().zip(borrowed_child.color.0.iter()) {
                    *own_color += u32::from(*child_color)
                } 
                self.pixel_count += borrowed_child.pixel_count;
                result += 1;
            }
            *child = None; 
        }

        result
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
