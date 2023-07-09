use fdt_rs::{
    base::{DevTree, DevTreeNode},
    index::{iters::DevTreeIndexNodeSiblingIter, DevTreeIndex, DevTreeIndexNode, DevTreeIndexProp},
    prelude::PropReader,
};

use crate::{debug::LogLevel, mem::phys::PhysicalMemoryRegion};

#[repr(C, align(0x10))]
struct FdtIndexBuffer([u8; 32768]);

static mut FDT_INDEX_BUFFER: FdtIndexBuffer = FdtIndexBuffer::zeroed();

impl FdtIndexBuffer {
    const fn zeroed() -> Self {
        Self([0; 32768])
    }
}

pub type TNode<'a> = DevTreeIndexNode<'a, 'a, 'a>;
pub type TProp<'a> = DevTreeIndexProp<'a, 'a, 'a>;

#[derive(Clone)]
pub struct FdtMemoryRegionIter<'a> {
    inner: DevTreeIndexNodeSiblingIter<'a, 'a, 'a>,
}

pub struct DeviceTree<'a> {
    tree: DevTree<'a>,
    index: DevTreeIndex<'a, 'a>,
}

impl<'a> DeviceTree<'a> {
    pub unsafe fn from_addr(virt: usize) -> Self {
        let tree = DevTree::from_raw_pointer(virt as _).unwrap();
        let index = DevTreeIndex::new(tree, &mut FDT_INDEX_BUFFER.0).unwrap();
        Self { tree, index }
    }

    pub fn node_by_path(&self, path: &str) -> Option<TNode> {
        find_node(self.index.root(), path.trim_start_matches('/'))
    }

    pub fn dump(&self, level: LogLevel) {
        dump_node(&self.index.root(), 0, level)
    }

    pub fn size(&self) -> usize {
        self.tree.totalsize()
    }
}

impl<'a> FdtMemoryRegionIter<'a> {
    pub fn new(dt: &'a DeviceTree) -> Self {
        let inner = dt.index.root().children();
        Self { inner }
    }
}

impl Iterator for FdtMemoryRegionIter<'_> {
    type Item = PhysicalMemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let Some(item) = self.inner.next() else {
                break None;
            };

            if item.name().unwrap_or("").starts_with("memory@") {
                let reg = item
                    .props()
                    .find(|p| p.name().unwrap_or("") == "reg")
                    .unwrap();

                break Some(PhysicalMemoryRegion {
                    base: reg.u64(0).unwrap() as usize,
                    size: reg.u64(1).unwrap() as usize,
                });
            }
        }
    }
}

pub fn find_prop<'a>(node: &TNode<'a>, name: &str) -> Option<TProp<'a>> {
    node.props().find(|p| p.name().unwrap_or("") == name)
}

fn path_component_left(path: &str) -> (&str, &str) {
    if let Some((left, right)) = path.split_once('/') {
        (left, right.trim_start_matches('/'))
    } else {
        (path, "")
    }
}

fn find_node<'a>(at: TNode<'a>, path: &str) -> Option<TNode<'a>> {
    let (item, path) = path_component_left(path);
    if item.is_empty() {
        assert_eq!(path, "");
        Some(at)
    } else {
        let child = at.children().find(|c| c.name().unwrap() == item)?;
        if path.is_empty() {
            Some(child)
        } else {
            find_node(child, path)
        }
    }
}

fn dump_node(node: &TNode, depth: usize, level: LogLevel) {
    fn indent(level: LogLevel, depth: usize) {
        for _ in 0..depth {
            log_print!(level, "  ");
        }
    }

    let node_name = node.name().unwrap();

    // Don't dump these
    if node_name.starts_with("virtio_mmio@") {
        return;
    }

    indent(level, depth);
    log_print!(level, "{:?} {{\n", node_name);
    for prop in node.props() {
        indent(level, depth + 1);
        let name = prop.name().unwrap();
        log_print!(level, "{name:?} = ");

        match name {
            "compatible" | "stdout-path" => log_print!(level, "{:?}", prop.str().unwrap()),
            _ => log_print!(level, "{:x?}", prop.raw()),
        }

        log_print!(level, "\n");
    }

    for child in node.children() {
        dump_node(&child, depth + 1, level);
    }

    indent(level, depth);
    log_print!(level, "}}\n");
}
