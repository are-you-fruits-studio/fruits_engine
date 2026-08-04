use std::{collections::{HashMap, HashSet}, hash::Hash};

#[derive(Clone)]
pub struct TreeBuilder<T> {
    roots: HashSet<T>,
    nodes: HashMap<T, Vec<T>>,
}

impl<T: Eq + Hash + Clone> TreeBuilder<T> {
    pub fn new() -> Self {
        Self {
            roots: Default::default(),
            nodes: Default::default(),
        }
    }

    pub fn insert_single(&mut self, node: T) -> bool {
        if self.nodes.contains_key(&node) {
            return false;
        }

        self.nodes.insert(node.clone(), Vec::new());
        self.roots.insert(node);

        true
    }
    
    pub fn insert_pair(&mut self, parent: T, child: T) -> bool {
        if parent == child {
            return self.insert_single(parent);
        }

        let is_child_root = self.roots.contains(&child);
        let does_contain_child = self.nodes.contains_key(&child);
        let does_contain_parent = self.nodes.contains_key(&parent);

        if is_child_root && !does_contain_parent {
            self.roots.remove(&child);
            self.roots.insert(parent.clone());
            self.nodes.insert(parent, vec![child]);
            return true;
        }

        if !does_contain_child && does_contain_parent {
            self.nodes.get_mut(&parent).unwrap().push(child.clone());
            self.nodes.insert(child, Vec::new());
            return true;
        }

        if !does_contain_child && !does_contain_parent {
            self.nodes.insert(child.clone(), Vec::new());
            self.nodes.insert(parent.clone(), vec![child]);
            self.roots.insert(parent);
            return true;
        }

        return false;
    }

    pub const fn roots(&self) -> &HashSet<T> {
        &self.roots
    }

    pub const fn nodes(&self) -> &HashMap<T, Vec<T>> {
        &self.nodes
    }

    pub fn build(mut self) -> Vec<TreeNode<T>> {
        let mut roots = Vec::with_capacity(self.roots.len());

        for root in self.roots {
            roots.push(Self::to_tree_node_recursive(&mut self.nodes, root));
        }

        roots
    }

    fn to_tree_node_recursive(nodes: &mut HashMap<T, Vec<T>>, parent: T) -> TreeNode<T> {
        TreeNode {
            children: nodes
                .remove(&parent)
                .into_iter()
                .flatten()
                .map(|c| Self::to_tree_node_recursive(nodes, c))
                .collect(),
            value: parent,
        }
    }
}

#[derive(Debug)]
pub struct TreeNode<T> {
    pub value: T,
    pub children: Vec<TreeNode<T>>,
}

impl<T> TreeNode<T> {
    pub fn iter_nodes_recursively(&mut self, mut f: impl FnMut(&mut TreeNode<T>)) {
        self.iter_nodes_recursively_internal(&mut f);
    }
    fn iter_nodes_recursively_internal(&mut self, f: &mut impl FnMut(&mut TreeNode<T>)) {
        f(self);
        for node in &mut self.children {
            node.iter_nodes_recursively_internal(f);
        }
    }
}