
use core::{borrow, fmt};
use std::{cell::RefCell, rc::Rc};

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

// turn all of these into weak pointers.
type LevelVec = [Vec<Rc<RefCell<OctreeNode>>>; MAX_DEPTH as usize];
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
    pub fn make_palette(&mut self, color_count: usize) {
        let palette = Vec::<Rgb<u8>>::new();
        let palette_index = 0;
        let mut leaf_count = self.get_leaf_nodes().len();

        for level in self.levels.iter_mut() {
            for node in level {
                let mut node = node.borrow_mut();
                leaf_count -= node.remove_leaves() as usize;
                todo!();
            }
        }
        // todo: levels.
    }
    pub fn add_color(&mut self, color: Rgb<u8>) {
        self.root.add_color(color, 0, &mut self.levels);
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
            levels[level].push(Rc::clone(&node));
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
        for node in self.children.iter() {
            if let Some(node) = node {
                let borrowed_node = node.borrow();
                if borrowed_node.is_leaf() {
                    leaf_nodes.push(Rc::clone(node)); // reference counted.
                } else {
                    for element in borrowed_node.get_leaf_nodes() {
                        leaf_nodes.push(element);
                    };
                }
            }
        }

        leaf_nodes
    }
    pub fn remove_leaves(&mut self) -> i32 {
        let mut result = 0;
        for ele in self.children.iter_mut() {
            if let Some(ele) = ele {
                let borrowed_node = ele.borrow_mut();
                for (a, b) in self.color.0.iter_mut().zip(borrowed_node.color.0.iter()) {
                    *a += u32::from(*b)
                } 
                self.pixel_count += borrowed_node.pixel_count;
                result += 1;
            }
            *ele = None; 
            // hopefully Rc doesn't have a reference in other code so
            //      Rust can deref this safely.
            //
            //  TODO: turn the levels array into weak pointers!
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
