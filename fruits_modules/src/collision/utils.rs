use crate::collision::{overlaps, CollisionAabb, CollisionShape};

// todo: recursion to iteration

#[derive(Default, Debug)]
pub struct Bvh<T> {
    root: Option<BvhNode<T>>,
}

#[derive(Debug)]
struct BvhNode<T> {
    aabb: CollisionAabb,
    core: BvhNodeCore<T>,
}

#[derive(Debug)]
enum BvhNodeCore<T> {
    Leaf(T),
    Branch(Box<[BvhNode<T>; 2]>),
}

impl<T> Bvh<T> {
    pub fn new(values: Vec<(CollisionAabb, T)>) -> Self {
        Self {
            root: BvhNode::new(values, 0),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<T: Clone> Bvh<T> {
    pub fn query(&self, query: CollisionShape, hits: &mut Vec<T>) {
        let Some(root) = &self.root else {
            return;
        };

        root.query(query, hits);
    }
}

impl<T> BvhNode<T> {
    fn new(mut values: Vec<(CollisionAabb, T)>, depth: usize) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        if values.len() == 1 {
            let (aabb, item) = values.pop().unwrap();

            return Some(Self {
                aabb: aabb,
                core: BvhNodeCore::Leaf(item),
            });
        }

        let axis = depth % 3;
        values.sort_by(|a, b| {
            let va = a.0.center[axis] - a.0.extents[axis];
            let vb = b.0.center[axis] - b.0.extents[axis];

            va.partial_cmp(&vb).unwrap()
        });

        let mid = values.len() / 2;
        let right = BvhNode::new(values.split_off(mid), depth + 1).unwrap();
        let left = BvhNode::new(values, depth + 1).unwrap();

        let aabb = left.aabb.merge(right.aabb);

        Some(BvhNode {
            aabb,
            core: BvhNodeCore::Branch(Box::new([left, right]))
        })
    }
}

impl<T: Clone> BvhNode<T> {
    fn query(&self, query: CollisionShape, hits: &mut Vec<T>) {
        if !overlaps(self.aabb.into(), query) {
            return;
        }
        
        match &self.core {
            BvhNodeCore::Leaf(id) => {
                hits.push(id.clone());
            },
            BvhNodeCore::Branch(children) => {
                for child in children.iter() {
                    child.query(query, hits);
                }
            }
        }
    }
}
