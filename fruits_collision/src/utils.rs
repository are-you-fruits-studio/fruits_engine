use fruits_ffi::{FfiBox, FfiOption};

use crate::{CollisionAabb, CollisionShape, overlaps};

// todo: recursion to iteration

/// Bounding-volume hierarchy mapping [`CollisionShape`]s to payloads of type `T`.
///
/// Built once from a set of shapes via [`new`](Self::new) and queried with
/// [`query`](Self::query). The tree is binary and median-split along a cycling axis,
/// giving roughly logarithmic broad-phase culling.
#[repr(C)]
#[derive(Default, Debug)]
pub struct Bvh<T: Clone> {
    /// Root node, or `None` when the hierarchy is empty.
    root: FfiOption<BvhNode<T>>,
}

/// A single node of a [`Bvh`]: a bounding [`CollisionAabb`] plus its [`BvhNodeCore`].
#[repr(C)]
#[derive(Debug)]
struct BvhNode<T: Clone> {
    /// Axis-aligned bounds enclosing this node's subtree.
    aabb: CollisionAabb,
    /// Whether the node is a leaf or an internal branch.
    core: BvhNodeCore<T>,
}

/// Payload of a [`BvhNode`]: either a stored shape or a pair of child nodes.
#[repr(C)]
#[derive(Debug)]
enum BvhNodeCore<T: Clone> {
    /// A leaf holding one shape and its associated payload.
    Leaf(CollisionShape, T),
    /// An internal node owning its two children.
    Branch(FfiBox<[BvhNode<T>; 2]>),
}

impl<T: Clone> Bvh<T> {
    /// Builds a hierarchy from an iterator of shapes and their payloads.
    ///
    /// Each shape's bounding box is precomputed (via [`CollisionShape::to_aabb`]) and the
    /// nodes are recursively median-split along the X/Y/Z axes in turn.
    pub fn new(values: impl Iterator<Item = (CollisionShape, T)>) -> Self {
        let mut values = values
            .map(|(s, t)| (s, s.to_aabb(), t))
            .collect::<Vec<_>>();

        Self {
            root: BvhNode::new(&mut values, 0).into(),
        }
    }

    /// Returns `true` if the hierarchy contains no shapes.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<T: Clone> Bvh<T> {
    /// Appends the payload of every stored shape overlapping `query` to `hits`.
    ///
    /// Descends only into subtrees whose bounding box overlaps `query`, then tests the
    /// exact leaf shapes with [`overlaps`]. Existing entries in `hits` are preserved.
    pub fn query(&self, query: CollisionShape, hits: &mut Vec<T>) {
        let Some(root) = self.root.as_ref() else {
            return;
        };

        root.query(query, hits);
    }
}

impl<T: Clone> BvhNode<T> {
    /// Recursively builds a subtree from `values`, splitting on the `depth % 3` axis.
    ///
    /// Returns `None` only when `values` is empty. `depth` selects the split axis and
    /// increases by one per level.
    fn new(values: &mut [(CollisionShape, CollisionAabb, T)], depth: usize) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        if values.len() == 1 {
            let (shape, aabb, item) = values[0].clone();

            return Some(Self {
                aabb,
                core: BvhNodeCore::Leaf(shape, item),
            });
        }

        let axis = depth % 3;
        values.sort_unstable_by(|a, b| {
            let va = a.1.center[axis] - a.1.extents[axis];
            let vb = b.1.center[axis] - b.1.extents[axis];

            va.partial_cmp(&vb).unwrap()
        });

        let mid = values.len() / 2;
        let (left, right) = values.split_at_mut(mid);
        let left = BvhNode::new(left, depth + 1).unwrap();
        let right = BvhNode::new(right, depth + 1).unwrap();

        let aabb = left.aabb.merge(right.aabb);

        Some(BvhNode {
            aabb,
            core: BvhNodeCore::Branch(FfiBox::new([left, right])),
        })
    }
}

impl<T: Clone> BvhNode<T> {
    /// Recursively collects payloads of leaf shapes overlapping `query` into `hits`.
    ///
    /// Prunes the subtree early when `query` misses this node's bounding box.
    fn query(&self, query: CollisionShape, hits: &mut Vec<T>) {
        if !overlaps(self.aabb.into(), query) {
            return;
        }

        match &self.core {
            BvhNodeCore::Leaf(shape, id) => {
                if overlaps(*shape, query) {
                    hits.push(id.clone());
                }
            }
            BvhNodeCore::Branch(children) => {
                for child in children.iter() {
                    child.query(query, hits);
                }
            }
        }
    }
}
