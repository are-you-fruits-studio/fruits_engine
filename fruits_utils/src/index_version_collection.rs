use std::{cmp::Ordering, collections::VecDeque};

pub struct VersionCollection<T> {
    items: Vec<DataWithVersion<T>>,
    free_places: VecDeque<u32>,
    // reserved_places: VecDeque<usize>,
    count: usize,
}
impl<T> Default for VersionCollection<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct VersionIndex {
    pub index: u32,
    pub version: u32,
}

impl Ord for VersionIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.index.cmp(&other.index) {
            Ordering::Equal => self.version.cmp(&other.version),
            cmp => cmp,
        }
    }
}

impl PartialOrd for VersionIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct DataWithVersion<T> {
    pub version: u32,
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
            let item = &mut self.items[index as usize];

            item.data = Some(data);
            
            self.count = self.count.checked_add(1).unwrap_or_else(|| panic!("VersionCollection count overflow"));

            return VersionIndex {
                index,
                version: item.version,
            }
        }

        let index: u32 = self.items.len().try_into().unwrap_or_else(|_| panic!("VersionCollection count overflow"));
        // entities with version 0 cannot exist.
        let version = 1;

        self.items.push(DataWithVersion::<T> {
            data: Some(data),
            version,
        });

        self.count = self.count.checked_add(1).unwrap_or_else(|| panic!("VersionCollection count overflow"));
        
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
        let data_with_version = self.items.get(index.index as usize)?;

        if index.version != data_with_version.version {
            return None;
        }

        Some(data_with_version.data.as_ref().unwrap())
    }

    pub fn get_mut(&mut self, index: VersionIndex) -> Option<&mut T> {
        self.get_data_with_version_mut(index).map(|d| d.data.as_mut().unwrap())
    }

    fn get_data_with_version_mut(&mut self, index: VersionIndex) -> Option<&mut DataWithVersion<T>> {
        let data_with_version = self.items.get_mut(index.index as usize)?;

        if index.version != data_with_version.version {
            return None;
        }

        Some(data_with_version)
    }

    pub fn contains_index(&self, index: VersionIndex) -> bool {
        let Some(data_with_version) = self.items.get(index.index as usize) else {
            return false;
        };

        index.version == data_with_version.version
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}