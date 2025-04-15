
use core::fmt;
use crate::core::accum_octree::get_color_index;
use std::{cell::RefCell, rc::Rc};
use image::Rgb;

#[derive(Clone, Debug)]
pub struct MortonOctreeNode {
    content: String,
    children: [Option<Rc<RefCell<MortonOctreeNode>>>; 8],
}
pub struct MortonOctree {
    depth: usize,
    root: MortonOctreeNode,
}

impl fmt::Display for MortonOctree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let node = &self.root;
        write!(f, "{}", node.to_string())
    }
}

impl fmt::Display for MortonOctreeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let children: Vec<String> = self.children.iter()
            .filter_map(|c| c.as_ref().map(|x| format!("{}", x.borrow())))
            .collect();
        write!(f, "<str: {}, children: [{}]>", self.content, children.join(", "))
    }
}

impl MortonOctree {
    pub fn new(depth: usize) -> Self {
        Self {
            depth,
            root: MortonOctreeNode::new("Comfy".to_string()),
        }
    }
    pub fn add_color(&mut self, color: Rgb<u8>) {
        self.root.add_node(color, 0, self.depth);
    }
    pub fn traverse(&self) -> String {
        let mut s = String::new();
        self.root.traverse(&mut s);

        s
    }
}

impl MortonOctreeNode {
    pub fn new(content: String) -> Self {
        Self {
            content,
            children: std::array::from_fn(|_| None),
        }
    }
    pub fn traverse(&self, out: &mut String) {
        for child in self.children.iter() {
            if let Some(child) = child {
                let child = child.as_ref().borrow();
                out.push_str(&child.content);
                child.traverse(out);
            }
        }
    }
    pub fn add_node(&mut self, color: Rgb<u8>, level: usize, depth: usize) {
        if level >= depth {
            return;
        }
        let index = get_color_index(color, level);
        let child = &mut self.children[index];
        if child.is_none() {
            let node = Rc::new(RefCell::new(MortonOctreeNode::new("test".to_string())));
            self.children[index] = Some(node);
        }
        let mut node = self.children[index].as_ref().unwrap().borrow_mut();
        node.add_node(color, level + 1, depth);
    }
}
