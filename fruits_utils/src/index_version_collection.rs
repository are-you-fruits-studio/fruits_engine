use std::{cmp::Ordering, collections::VecDeque};

pub struct VersionCollection<T> {
    items: Vec<DataWithVersion<T>>,
    free_places: VecDeque<usize>,
    // reserved_places: VecDeque<usize>,
    count: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VersionIndex {
    pub index: usize,
    pub version: usize,
}

impl Ord for VersionIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.index.cmp(&other.index) {
            Ordering::Equal => self.version.cmp(&other.version),
            cmp @ _ => cmp,
        }
    }
}

impl PartialOrd for VersionIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct DataWithVersion<T> {
    pub version: usize,
    pub data: Option<T>,
}

impl<T> VersionCollection<T> {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            free_places: VecDeque::new(),
            count: 0,
        }
    }

    pub fn insert(&mut self, data: T) -> VersionIndex {
        if let Some(index) = self.free_places.pop_front() {
            let version = self.items[index].version;

            self.items[index] = DataWithVersion::<T> {
                data: Some(data),
                version,
            };
            
            self.count += 1;

            return VersionIndex {
                index,
                version,
            }
        }

        let index: usize = self.items.len();
        // entities with version 0 cannot exist.
        let version = 1;

        self.items.push(DataWithVersion::<T> {
            data: Some(data),
            version,
        });

        self.count += 1;
        
        VersionIndex {
            index,
            version,
        }
    }

    pub fn remove(&mut self, index: VersionIndex) -> Option<T> {
        let data_with_version = self.get_data_with_version_mut(index)?;

        data_with_version.version = data_with_version.version.wrapping_add(1);

        // to prevent version = 0.
        if data_with_version.version == 0 {
            data_with_version.version += 1;
        }

        let data = data_with_version.data.take().unwrap();

        self.free_places.push_back(index.index);

        self.count -= 1;

        Some(data)
    }

    pub fn get(&self, index: VersionIndex) -> Option<&T> {
        let data_with_version = self.items.get(index.index)?;

        if index.version != data_with_version.version {
            return None;
        }

        Some(data_with_version.data.as_ref().unwrap())
    }

    pub fn get_mut(&mut self, index: VersionIndex) -> Option<&mut T> {
        self.get_data_with_version_mut(index).map(|d| d.data.as_mut().unwrap())
    }

    fn get_data_with_version_mut(&mut self, index: VersionIndex) -> Option<&mut DataWithVersion<T>> {
        let data_with_version = self.items.get_mut(index.index)?;

        if index.version != data_with_version.version {
            return None;
        }

        Some(data_with_version)
    }

    pub fn contains_index(&self, index: VersionIndex) -> bool {
        let Some(data_with_version) = self.items.get(index.index) else {
            return false;
        };

        index.version == data_with_version.version
    }

    pub fn len(&self) -> usize {
        self.count
    }
}