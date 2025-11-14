use fruits_ffi::{FfiBox, FfiOption};

use crate::collision::{CollisionAabb, CollisionShape, overlaps};

// todo: recursion to iteration

#[repr(C)]
#[derive(Default, Debug)]
pub struct Bvh<T> {
    root: FfiOption<BvhNode<T>>,
}

#[repr(C)]
#[derive(Debug)]
struct BvhNode<T> {
    aabb: CollisionAabb,
    core: BvhNodeCore<T>,
}

#[repr(C)]
#[derive(Debug)]
enum BvhNodeCore<T> {
    Leaf(CollisionShape, T),
    Branch(FfiBox<[BvhNode<T>; 2]>),
}

impl<T> Bvh<T> {
    pub fn new(values: Vec<(CollisionShape, T)>) -> Self {
        Self {
            root: BvhNode::new(values, 0).into(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<T: Clone> Bvh<T> {
    pub fn query(&self, query: CollisionShape, hits: &mut Vec<T>) {
        let Some(root) = self.root.as_ref() else {
            return;
        };

        root.query(query, hits);
    }
}

impl<T> BvhNode<T> {
    fn new(mut values: Vec<(CollisionShape, T)>, depth: usize) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        if values.len() == 1 {
            let (shape, item) = values.pop().unwrap();

            return Some(Self {
                aabb: shape.to_aab(),
                core: BvhNodeCore::Leaf(shape, item),
            });
        }

        let axis = depth % 3;
        values.sort_by(|a, b| {
            let aab_a = a.0.to_aab();
            let aab_b = b.0.to_aab();

            let va = aab_a.center[axis] - aab_a.extents[axis];
            let vb = aab_b.center[axis] - aab_b.extents[axis];

            va.partial_cmp(&vb).unwrap()
        });

        let mid = values.len() / 2;
        let right = BvhNode::new(values.split_off(mid), depth + 1).unwrap();
        let left = BvhNode::new(values, depth + 1).unwrap();

        let aabb = left.aabb.merge(right.aabb);

        Some(BvhNode {
            aabb,
            core: BvhNodeCore::Branch(FfiBox::new([left, right])),
        })
    }
}

impl<T: Clone> BvhNode<T> {
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
