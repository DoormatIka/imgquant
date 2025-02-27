
use image::Rgba;

fn get_color_index(color: Rgba<u8>, level: usize) -> usize {
    let mut index: usize = 0;
    let mask = 0b10000000 >> level;
    if color.0[0] & mask != 0 { index |= 0b100 }
    if color.0[1] & mask != 0 { index |= 0b010 }
    if color.0[2] & mask != 0 { index |= 0b001 }

    return index;
}

struct OctreeNode {
    color: Rgba<u8>,
    pixel_count: u32,
    // indirection kills performance.
    children: [Box<Option<OctreeNode>>; 8],
}
impl OctreeNode {
    fn is_leaf() {
        
    }
}

struct Octree {
    root: OctreeNode,
    levels: Vec<Vec<OctreeNode>>,
}
impl Octree {
    fn add_color(self, color: Rgba<u8>) {

    }
}

pub fn test() {

}
