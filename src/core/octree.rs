
use core::fmt;
use std::{cell::RefCell, ops::{AddAssign, Sub, Mul, Div}, rc::{Rc, Weak}};

use image::Rgb;
// note: 0, 1, 2 corresponds to R, G, B

pub fn get_color_index(color: Rgb<u8>, level: usize) -> usize {
    let mut index: usize = 0;
    let mask = 0b10000000 >> level;
    if color.0[0] & mask != 0 { index |= 0b100; }
    if color.0[1] & mask != 0 { index |= 0b010; }
    if color.0[2] & mask != 0 { index |= 0b001; }

    return index;
}
pub fn add_colors<T, U>(color: &mut Rgb<T>, other_color: &Rgb<U>) 
where
    T: AddAssign + From<U>,
    U: Copy,
{
    color.0.iter_mut()
        .zip(other_color.0.iter().copied())
        .for_each(|(a, b)| *a += T::from(b));
}

pub fn sub_colors<T, U>(color: &Rgb<T>, other_color: &Rgb<U>) -> Rgb<T> 
where
    T: Sub<U, Output = T> + Copy,
    U: Copy,
{
    Rgb([
        color.0[0] - other_color.0[0],
        color.0[1] - other_color.0[1],
        color.0[2] - other_color.0[2],
    ])
}

pub fn mul_colors<T, U>(color: &Rgb<T>, other_color: &Rgb<U>) -> Rgb<T> 
where
    T: Mul<U, Output = T> + Copy,
    U: Copy,
{
    Rgb([
        color.0[0] * other_color.0[0],
        color.0[1] * other_color.0[1],
        color.0[2] * other_color.0[2],
    ])
}

pub fn div_colors<T, U>(color: &Rgb<T>, other_color: &Rgb<U>) -> Rgb<T> 
where
    T: Div<U, Output = T> + Copy,
    U: Copy,
{
    Rgb([
        color.0[0] / other_color.0[0],
        color.0[1] / other_color.0[1],
        color.0[2] / other_color.0[2],
    ])
}

const MAX_DEPTH: u8 = 6;

#[derive(Clone, Debug)]
pub struct OctreeNode {
    children: [Option<Rc<RefCell<OctreeNode>>>; 8],
    color: Rgb<u32>,
    pixel_count: u32,
    palette_index: u32,
    error_value: u32,
}

type LevelVec = [Vec<Weak<RefCell<OctreeNode>>>; MAX_DEPTH as usize];
pub struct Octree {
    root: OctreeNode,
    levels: LevelVec,
}

impl Octree {
    pub fn new() -> Self {
        Self {
            root: OctreeNode::new(),
            levels: std::array::from_fn(|_| {
                let mut v = Vec::new();
                v.reserve(10_000);
                v
            }),
        }
    }
    pub fn make_palette(&mut self, color_count: i32) -> Vec<Rgb<u8>> {
        let mut palette = Vec::<Rgb<u8>>::new();
        let mut palette_index = 0;
        let leaves = self.get_leaf_nodes();
        let mut leaf_count = leaves.len() as i32;

        for level_index in (0..(MAX_DEPTH as usize - 1)).rev() {
            let level = &mut self.levels[level_index];
            for node in level {
                match node.upgrade() {
                    Some(node) => {
                        let mut node = node.borrow_mut();
                        leaf_count -= node.remove_leaves();
                    },
                    None => {

                    }
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
                        let rgb = Rgb::<u8>([
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
    pub fn add_color(&mut self, color: Rgb<u8>) {
        self.root.add_color(color, 0, &mut self.levels);
    }
    /// Returns the palette index for the closest color in the octree to your given color.
    /// 
    /// # Arguments
    /// * `color` - The color value to find the palette index of.
    /// * `ignore_no_color` - If `true`, the function will 
    /// skip colors that are not accounted for in the octree.
    ///
    /// # Returns
    /// * `Some(index)` if a suitable match is found.
    /// * `None` if no match is found.
    pub fn get_palette_index(&self, color: Rgb<u8>, ignore_no_color: bool) -> Option<usize> {
        self.root.get_palette_index(color, 0, ignore_no_color)
    }
    pub fn get_leaf_nodes(&self) -> Vec<Weak<RefCell<OctreeNode>>> {
        let leaves = self.root.get_leaf_nodes();
        return leaves;
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
            error_value: 0,
        }
    }
    pub fn add_color(&mut self, color: Rgb<u8>, level: usize, levels: &mut LevelVec) {
        if level >= usize::from(MAX_DEPTH) {
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
        node.add_color(color, level + 1, levels);
    }
    pub fn get_palette_index(&self, color: Rgb<u8>, level: usize, ignore_no_color: bool) -> Option<usize> {
        if self.is_leaf() {
            return Some(self.palette_index as usize);
        } else {
            let index = get_color_index(color, level);
            match &self.children[index] {
                Some(cell) => {
                    let c = cell.borrow();
                    return c.get_palette_index(color, level + 1, ignore_no_color);
                },
                None => {
                    if ignore_no_color {
                        for node in self.children.iter() {
                            if let Some(node) = node {
                                let c = node.borrow();
                                return c.get_palette_index(color, level + 1, ignore_no_color);
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



