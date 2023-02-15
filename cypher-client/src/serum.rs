use {
    anchor_spl::dex::serum_dex::state::{
        Event, EventQueueHeader, QueueHeader, ACCOUNT_HEAD_PADDING, ACCOUNT_TAIL_PADDING,
    },
    arrayref::array_refs,
    bytemuck::{cast_mut, cast_ref, cast_slice, from_bytes, Pod, Zeroable},
    num_enum::{IntoPrimitive, TryFromPrimitive},
    safe_transmute::{
        guard::SingleManyGuard, to_bytes::transmute_to_bytes, transmute_many,
        transmute_many_pedantic, transmute_one_pedantic,
    },
    static_assertions::const_assert_eq,
    std::{
        borrow::Cow,
        convert::TryFrom,
        mem::{align_of, size_of},
    },
};

pub fn parse_dex_event_queue(data_words: &[u64]) -> (EventQueueHeader, &[Event], &[Event]) {
    let (header_words, event_words) = data_words.split_at(size_of::<EventQueueHeader>() >> 3);
    let header: EventQueueHeader = transmute_one_pedantic(transmute_to_bytes(header_words))
        .map_err(|e| e.without_src())
        .unwrap();
    let events: &[Event] = transmute_many::<_, SingleManyGuard>(transmute_to_bytes(event_words))
        .map_err(|e| e.without_src())
        .unwrap();
    let (tail_seg, head_seg) = events.split_at(header.head() as usize);
    let head_len = head_seg.len().min(header.count() as usize);
    let tail_len = header.count() as usize - head_len;
    (header, &head_seg[..head_len], &tail_seg[..tail_len])
}

pub fn remove_dex_account_padding<'a>(data: &'a [u8]) -> Cow<'a, [u64]> {
    let head = &data[..ACCOUNT_HEAD_PADDING.len()];
    if data.len() < ACCOUNT_HEAD_PADDING.len() + ACCOUNT_TAIL_PADDING.len() {
        panic!();
    }
    if head != ACCOUNT_HEAD_PADDING {
        panic!();
    }
    let tail = &data[data.len() - ACCOUNT_TAIL_PADDING.len()..];
    if tail != ACCOUNT_TAIL_PADDING {
        panic!();
    }
    let inner_data_range = ACCOUNT_HEAD_PADDING.len()..(data.len() - ACCOUNT_TAIL_PADDING.len());
    let inner: &'a [u8] = &data[inner_data_range];
    let words: Cow<'a, [u64]> = match transmute_many_pedantic::<u64>(inner) {
        Ok(word_slice) => Cow::Borrowed(word_slice),
        Err(transmute_error) => {
            let word_vec = transmute_error.copy().map_err(|e| e.without_src()).unwrap();
            Cow::Owned(word_vec)
        }
    };
    words
}

pub fn parse_dex_account<T: Pod>(data: &[u8]) -> T {
    let data_len = data.len() - 12;
    let (_, rest) = data.split_at(5);
    let (mid, _) = rest.split_at(data_len);
    *from_bytes(mid)
}

pub type NodeHandle = u32;

#[derive(IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
enum NodeTag {
    Uninitialized = 0,
    InnerNode = 1,
    LeafNode = 2,
    FreeNode = 3,
    LastFreeNode = 4,
}

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
struct InnerNode {
    tag: u32,           // 4
    prefix_len: u32,    // 8
    key: u128,          // 24
    children: [u32; 2], // 32
    _padding: [u64; 5], // 72
}

unsafe impl Zeroable for InnerNode {}
unsafe impl Pod for InnerNode {}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(packed)]
pub struct LeafNode {
    tag: u32,             // 4
    owner_slot: u8,       // 5
    fee_tier: u8,         // 6
    padding: [u8; 2],     // 8
    key: u128,            // 24
    owner: [u64; 4],      // 56
    quantity: u64,        // 64
    client_order_id: u64, // 72
}

unsafe impl Zeroable for LeafNode {}
unsafe impl Pod for LeafNode {}

impl LeafNode {
    #[inline]
    pub fn price(&self) -> u64 {
        (self.key >> 64) as u64
    }

    #[inline]
    pub fn order_id(&self) -> u128 {
        self.key
    }

    #[inline]
    pub fn quantity(&self) -> u64 {
        self.quantity
    }

    #[inline]
    pub fn set_quantity(&mut self, quantity: u64) {
        self.quantity = quantity;
    }

    #[inline]
    pub fn owner(&self) -> [u64; 4] {
        self.owner
    }

    #[inline]
    pub fn owner_slot(&self) -> u8 {
        self.owner_slot
    }

    #[inline]
    pub fn client_order_id(&self) -> u64 {
        self.client_order_id
    }
}

#[derive(Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
struct FreeNode {
    tag: u32,
    next: u32,
    _padding: [u64; 8],
}
unsafe impl Zeroable for FreeNode {}
unsafe impl Pod for FreeNode {}

const _INNER_NODE_SIZE: usize = size_of::<InnerNode>();
const _LEAF_NODE_SIZE: usize = size_of::<LeafNode>();
const _FREE_NODE_SIZE: usize = size_of::<FreeNode>();
const _NODE_SIZE: usize = 72;

const _INNER_NODE_ALIGN: usize = align_of::<InnerNode>();
const _LEAF_NODE_ALIGN: usize = align_of::<LeafNode>();
const _FREE_NODE_ALIGN: usize = align_of::<FreeNode>();
const _NODE_ALIGN: usize = 1;

const_assert_eq!(_NODE_SIZE, _INNER_NODE_SIZE);
const_assert_eq!(_NODE_SIZE, _LEAF_NODE_SIZE);
const_assert_eq!(_NODE_SIZE, _FREE_NODE_SIZE);

const_assert_eq!(_NODE_ALIGN, _INNER_NODE_ALIGN);
const_assert_eq!(_NODE_ALIGN, _LEAF_NODE_ALIGN);
const_assert_eq!(_NODE_ALIGN, _FREE_NODE_ALIGN);

#[derive(Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
pub struct AnyNode {
    tag: u32,
    data: [u32; 17],
}
unsafe impl Zeroable for AnyNode {}
unsafe impl Pod for AnyNode {}

enum NodeRef<'a> {
    Inner(&'a InnerNode),
    Leaf(&'a LeafNode),
}

enum NodeRefMut<'a> {
    Inner(&'a mut InnerNode),
    Leaf(&'a mut LeafNode),
}

impl AnyNode {
    fn case(&self) -> Option<NodeRef> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRef::Inner(cast_ref(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRef::Leaf(cast_ref(self))),
            _ => None,
        }
    }

    fn case_mut(&mut self) -> Option<NodeRefMut> {
        match NodeTag::try_from(self.tag) {
            Ok(NodeTag::InnerNode) => Some(NodeRefMut::Inner(cast_mut(self))),
            Ok(NodeTag::LeafNode) => Some(NodeRefMut::Leaf(cast_mut(self))),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf(&self) -> Option<&LeafNode> {
        match self.case() {
            Some(NodeRef::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }

    #[inline]
    pub fn as_leaf_mut(&mut self) -> Option<&mut LeafNode> {
        match self.case_mut() {
            Some(NodeRefMut::Leaf(leaf_ref)) => Some(leaf_ref),
            _ => None,
        }
    }
}

impl AsRef<AnyNode> for InnerNode {
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}

impl AsRef<AnyNode> for LeafNode {
    #[inline]
    fn as_ref(&self) -> &AnyNode {
        cast_ref(self)
    }
}

const_assert_eq!(_NODE_SIZE, size_of::<AnyNode>());
const_assert_eq!(_NODE_ALIGN, align_of::<AnyNode>());

#[derive(Debug, Copy, Clone)]
#[repr(packed)]
#[allow(dead_code)]
pub struct SlabHeader {
    bump_index: u64,     // 8
    free_list_len: u64,  // 16
    free_list_head: u32, // 20
    root_node: u32,      // 24
    pub leaf_count: u64, // 32
}
unsafe impl Zeroable for SlabHeader {}
unsafe impl Pod for SlabHeader {}

const SLAB_HEADER_LEN: usize = size_of::<SlabHeader>();

#[cfg(debug_assertions)]
unsafe fn invariant(check: bool) {
    if check {
        unreachable!();
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
unsafe fn invariant(check: bool) {
    if check {
        std::hint::unreachable_unchecked();
    }
}

#[repr(transparent)]
pub struct Slab([u8]);

impl Slab {
    /// Creates a slab that holds and references the bytes
    ///
    /// ```compile_fail
    /// let slab = {
    ///     let mut bytes = [10; 100];
    ///     serum_dex::critbit::Slab::new(&mut bytes)
    /// };
    /// ```
    #[inline]
    pub fn new(bytes: &mut [u8]) -> &mut Self {
        let len_without_header = bytes.len().checked_sub(SLAB_HEADER_LEN).unwrap();
        let slop = len_without_header % size_of::<AnyNode>();
        let truncated_len = bytes.len() - slop;
        let bytes = &mut bytes[..truncated_len];
        let slab: &mut Self = unsafe { &mut *(bytes as *mut [u8] as *mut Slab) };
        slab.check_size_align(); // check alignment
        slab
    }

    /// Gets the root node.
    pub fn root(&self) -> Option<NodeHandle> {
        if self.header().leaf_count == 0 {
            return None;
        }

        Some(self.header().root_node)
    }

    // Each one of these does a preorder traversal
    pub fn get_depth(&self, depth: u64, is_asks: bool) -> Vec<&LeafNode> {
        let (header, _) = self.parts();
        let depth_to_get: usize = std::cmp::min(depth, header.leaf_count) as usize;

        let maybe_leafs = self.get_leaf_depth(depth_to_get, is_asks);
        match maybe_leafs {
            Some(l) => l,
            _ => Vec::new(),
        }
    }

    /// Gets leaf nodes up to a certain depth.
    fn get_leaf_depth(&self, depth: usize, asc: bool) -> Option<Vec<&LeafNode>> {
        let root: NodeHandle = self.root()?;
        let mut stack: Vec<NodeHandle> = Vec::with_capacity(self.header().leaf_count as usize);
        let mut res: Vec<&LeafNode> = Vec::with_capacity(depth);
        stack.push(root);
        loop {
            if stack.is_empty() {
                break;
            }
            let node_contents = self.get(stack.pop().unwrap()).unwrap();
            match node_contents.case().unwrap() {
                NodeRef::Inner(&InnerNode { children, .. }) => {
                    if asc {
                        stack.push(children[1]);
                        stack.push(children[0]);
                    } else {
                        stack.push(children[0]);
                        stack.push(children[1]);
                    }
                    continue;
                }
                NodeRef::Leaf(leaf) => {
                    res.push(leaf);
                }
            }
            if res.len() == depth {
                break;
            }
        }
        Some(res)
    }

    #[allow(clippy::ptr_offset_with_cast)]
    fn check_size_align(&self) {
        let (header_bytes, nodes_bytes) = array_refs![&self.0, SLAB_HEADER_LEN; .. ;];
        let _header: &SlabHeader = cast_ref(header_bytes);
        let _nodes: &[AnyNode] = cast_slice(nodes_bytes);
    }

    #[allow(clippy::ptr_offset_with_cast)]
    fn parts(&self) -> (&SlabHeader, &[AnyNode]) {
        unsafe {
            invariant(self.0.len() < size_of::<SlabHeader>());
            invariant((self.0.as_ptr() as usize) % align_of::<SlabHeader>() != 0);
            invariant(
                ((self.0.as_ptr() as usize) + size_of::<SlabHeader>()) % align_of::<AnyNode>() != 0,
            );
        }

        let (header_bytes, nodes_bytes) = array_refs![&self.0, SLAB_HEADER_LEN; .. ;];
        let header = cast_ref(header_bytes);
        let nodes = cast_slice(nodes_bytes);
        (header, nodes)
    }

    pub fn header(&self) -> &SlabHeader {
        self.parts().0
    }

    pub fn nodes(&self) -> &[AnyNode] {
        self.parts().1
    }
}

pub trait SlabView<T> {
    fn get(&self, h: NodeHandle) -> Option<&T>;
}

impl SlabView<AnyNode> for Slab {
    fn get(&self, key: u32) -> Option<&AnyNode> {
        let node = self.nodes().get(key as usize)?;
        let tag = NodeTag::try_from(node.tag);
        match tag {
            Ok(NodeTag::InnerNode) | Ok(NodeTag::LeafNode) => Some(node),
            _ => None,
        }
    }
}
