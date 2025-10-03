use std::collections::VecDeque;

pub struct OrderGraph {
    directions: Vec<Vec<usize>>,
    directors_count: Vec<usize>,
    initial_nodes: Vec<usize>,
}

impl OrderGraph {
    pub fn new(
        directions: Vec<Vec<usize>>,
    ) -> Option<Self> {
        let mut directors_count = vec![0_usize; directions.len()];

        for (src, dst) in directions.iter().enumerate() {
            for &directed_node in dst.iter() {
                if directed_node == src {
                    return None;
                }

                directors_count[directed_node] += 1;
            }
            if dst.contains(&src) {
                return None;
            }
        }

        let initial_nodes = directors_count.iter().enumerate().filter(|(_, c)| **c == 0).map(|(i, _)| i).collect();

        // todo: Add graph validation.

        Some(Self {
            directions,
            directors_count,
            initial_nodes,
        })
    }

    pub fn iter(&self) -> OrderGraphIterator {
        OrderGraphIterator::new(self)
    }
}

pub struct OrderGraphIterator {
    directions: Vec<Vec<usize>>,
    queue: VecDeque<usize>,
    unvisited_directors_count: Vec<usize>,
    processing_count: usize,
}

impl OrderGraphIterator {
    pub fn new(graph: &OrderGraph) -> Self {
        let mut queue = VecDeque::new();

        for initial_node in graph.initial_nodes.iter() {
            queue.push_back(*initial_node);
        }

        let unvisited_directors_count = graph.directors_count.clone();

        Self {
            directions: graph.directions.clone(),
            queue,
            unvisited_directors_count,
            processing_count: 0,
        }
    }

    pub fn start_next(&mut self) -> Option<usize> {
        let node = self.queue.pop_front()?;

        self.processing_count += 1;

        Some(node)
    }

    pub fn end(&mut self, node: usize) {
        self.processing_count -= 1;

        for direction in self.directions[node].iter() {
            let direction_directors_count = &mut self.unvisited_directors_count[*direction];
    
            *direction_directors_count -= 1;

            if *direction_directors_count == 0 {
                self.queue.push_back(*direction);
            } 
        }
    }

    pub fn all_started(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn all_ended(&self) -> bool {
        self.all_started() && self.processing_count == 0
    }
}