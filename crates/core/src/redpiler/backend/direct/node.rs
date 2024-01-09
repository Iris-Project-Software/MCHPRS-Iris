use crate::redpiler::block_is_dot;
use mchprs_blocks::blocks::{Block, ComparatorMode};
use smallvec::SmallVec;
use std::ops::{Index, IndexMut};

#[derive(Debug, Copy, Clone)]
pub struct NodeId(u32);

impl NodeId {
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Safety: index must be within bounds of nodes array
    pub unsafe fn from_index(index: usize) -> NodeId {
        NodeId(index as u32)
    }
}

// This is Pretty Bad:tm: because one can create a NodeId using another instance of Nodes,
// but at least some type system protection is better than none.
#[derive(Default)]
pub struct Nodes {
    pub nodes: Box<[Node]>,
}

impl Nodes {
    pub fn new(nodes: Box<[Node]>) -> Nodes {
        Nodes { nodes }
    }

    pub fn get(&self, idx: usize) -> NodeId {
        if self.nodes.get(idx).is_some() {
            NodeId(idx as u32)
        } else {
            panic!("node index out of bounds: {}", idx)
        }
    }

    pub fn inner(&self) -> &[Node] {
        &self.nodes
    }

    pub fn inner_mut(&mut self) -> &mut [Node] {
        &mut self.nodes
    }

    pub fn into_inner(self) -> Box<[Node]> {
        self.nodes
    }
}

impl Index<NodeId> for Nodes {
    type Output = Node;

    // The index here MUST have been created by this instance, otherwise scary things will happen !
    fn index(&self, index: NodeId) -> &Self::Output {
        unsafe { self.nodes.get_unchecked(index.0 as usize) }
    }
}

impl IndexMut<NodeId> for Nodes {
    fn index_mut(&mut self, index: NodeId) -> &mut Self::Output {
        unsafe { self.nodes.get_unchecked_mut(index.0 as usize) }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ForwardLink {
    data: u32,
}

impl ForwardLink {
    pub fn new(id: NodeId, side: bool, ss: u8) -> Self {
        assert!(id.index() < (1 << 27));
        // the clamp_weights compile pass should ensure ss < 16
        assert!(ss < 16);
        Self {
            data: (id.index() as u32) << 5 | if side { 1 << 4 } else { 0 } | ss as u32,
        }
    }

    pub fn node(self) -> NodeId {
        unsafe {
            // safety: ForwardLink is constructed using a NodeId
            NodeId::from_index((self.data >> 5) as usize)
        }
    }

    pub fn side(self) -> bool {
        self.data & (1 << 4) != 0
    }

    pub fn ss(self) -> u8 {
        (self.data & 0b1111) as u8
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    Repeater(u8),
    /// A non-locking repeater
    SimpleRepeater(u8),
    Torch,
    Comparator(ComparatorMode),
    Lamp,
    Button,
    Lever,
    PressurePlate,
    Trapdoor,
    Wire,
    Constant,
}

impl NodeType {
    pub fn is_io_block(self, block: Block) -> bool {
        if block_is_dot(block) {
            return true;
        }

        matches!(
            self,
            NodeType::Lamp
                | NodeType::Button
                | NodeType::Lever
                | NodeType::Trapdoor
                | NodeType::PressurePlate
        )
    }
}

#[repr(align(16))]
#[derive(Debug, Clone, Default)]
pub struct NodeInput {
    pub ss_counts: [u8; 16],
}

// struct is 128 bytes to fit nicely into cachelines
// which are usualy 64 bytes, it can vary but is almost always a power of 2
#[derive(Debug, Clone)]
#[repr(align(128))]
pub struct Node {
    pub ty: NodeType,
    pub default_inputs: NodeInput,
    pub side_inputs: NodeInput,
    pub updates: SmallVec<[ForwardLink; 18]>,

    pub facing_diode: bool,
    pub comparator_far_input: Option<u8>,

    /// Powered or lit
    pub powered: bool,
    /// Only for repeaters
    pub locked: bool,
    pub output_power: u8,
    pub changed: bool,
    pub pending_tick: bool,
}
