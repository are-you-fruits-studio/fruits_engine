use std::{collections::{HashMap, HashSet}, hash::Hash};

pub struct Graph<T: Eq + Hash + Copy + Clone>
{
    forward: HashMap<T, HashSet<T>>,
    backward: HashMap<T, HashSet<T>>,
    nodes: HashSet<T>,
}
impl<T: Eq + Hash + Copy + Clone> Graph<T> {
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            backward: HashMap::new(),
            nodes: HashSet::new(),
        }
    }

    pub fn insert_link(&mut self, source: T, destination: T) -> bool {
        if !self.nodes.insert(source) && !self.nodes.insert(destination)
        {
            return false;
        }
        
        if !self.forward.entry(source).or_default().insert(destination)
        {
            return false;
        }

        self.backward.entry(destination).or_default().insert(source);

        return true;
    }

    pub fn insert_node(&mut self, node: T) -> bool {
        return self.nodes.insert(node);
    }

    pub fn to_vec(&self) -> Vec<T> {
        return Self::to_list_internal(&self.backward, &self.nodes);
    }

    pub fn to_vec_rev(&self) -> Vec<T> {
        return Self::to_list_internal(&self.forward, &self.nodes);
    }

    fn to_list_internal(inverted: &HashMap<T, HashSet<T>>, nodes: &HashSet<T>) -> Vec<T> {
        let mut max_to_min = inverted.clone();

        let mut ordered_set = HashSet::<T>::new();
        let mut ordered = Vec::<T>::new();

        while max_to_min.len() != 0 {
            let (min, max) = Self::most_min(&max_to_min);

            if ordered_set.insert(min) {
                ordered.push(min);
            }

            if let Some(mins) = max_to_min.get_mut(&max) {
                mins.remove(&min);
                
                if mins.len() == 0
                {
                    if ordered_set.insert(max)
                    {
                        ordered.push(max);
                    }

                    max_to_min.remove(&max);
                }
            }
        }
        
        for node in nodes {
            if ordered_set.insert(*node) {
                ordered.push(*node);
            }
        }

        return ordered;
    }

    fn most_min(max_to_min: &HashMap<T, HashSet<T>>) -> (T, T)
    {
        let mut visited = HashSet::<T>::new();

        let (mut max, mut mins) = max_to_min.iter().next().unwrap();

        while visited.insert(*max)
        {
            let min = mins.iter().next().unwrap();

            let Some(new_mins) = max_to_min.get(min) else {
                return (*min, *max);
            };

            mins = new_mins;
            max = min;
        }

        panic!("The orderer contains circular dependencies. Cycle contains {} elements.", visited.len());
    }
}