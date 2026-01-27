use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

#[derive(Default)]
pub struct Graph<T: Eq + Hash + Clone> {
    forward: HashMap<T, HashSet<T>>,
    backward: HashMap<T, HashSet<T>>,
    nodes: HashSet<T>,
}
impl<T: Eq + Hash + Clone> Graph<T> {
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            backward: HashMap::new(),
            nodes: HashSet::new(),
        }
    }

    pub fn insert_edge(&mut self, src: T, dst: T) {
        self.forward.entry(src.clone()).or_default().insert(dst.clone());
        self.backward.entry(dst).or_default().insert(src);
    }

    pub fn insert_node(&mut self, node: T) -> bool {
        self.nodes.insert(node)
    }

    pub fn edges_from(&self, node: &T) -> Option<&HashSet<T>> {
        self.forward.get(node)
    }

    pub fn edges_to(&self, node: &T) -> Option<&HashSet<T>> {
        self.backward.get(node)
    }

    pub fn to_vec(&self) -> Result<Vec<T>, HashSet<T>> {
        Self::to_vec_internal(&self.backward, &self.nodes)
    }

    pub fn to_vec_rev(&self) -> Result<Vec<T>, HashSet<T>> {
        Self::to_vec_internal(&self.forward, &self.nodes)
    }

    fn to_vec_internal(inverted: &HashMap<T, HashSet<T>>, nodes: &HashSet<T>) -> Result<Vec<T>, HashSet<T>> {
        let mut max_to_min = inverted.clone();

        let mut ordered_set = HashSet::<T>::new();
        let mut ordered = Vec::<T>::new();

        while !max_to_min.is_empty() {
            let (min, max) = Self::most_min(&max_to_min)?;

            if ordered_set.insert(min.clone()) {
                ordered.push(min.clone());
            }

            if let Some(mins) = max_to_min.get_mut(&max) {
                mins.remove(&min);

                if mins.is_empty() {
                    if ordered_set.insert(max.clone()) {
                        ordered.push(max.clone());
                    }

                    max_to_min.remove(&max);
                }
            }
        }

        for node in nodes {
            if ordered_set.insert(node.clone()) {
                ordered.push(node.clone());
            }
        }

        Ok(ordered)
    }

    fn most_min(max_to_min: &HashMap<T, HashSet<T>>) -> Result<(T, T), HashSet<T>> {
        let mut visited = HashSet::<T>::new();

        let (mut max, mut mins) = max_to_min.iter().next().unwrap();

        while visited.insert(max.clone()) {
            let min = mins.iter().next().unwrap();

            let Some(new_mins) = max_to_min.get(min) else {
                return Ok((min.clone(), max.clone()));
            };

            mins = new_mins;
            max = min;
        }

        Err(visited)
    }
}
