use std::collections::{HashMap, HashSet};

use crate::entity::{unique_components_set::UniqueComponentsSet, archetype_unsafe::ArchetypeUnsafeFfi};



#[derive(Default)]
pub struct ArchetypesHolder {
    archetype_id_by_components: HashMap<UniqueComponentsSet, usize>,
    archetype_ids_by_component: HashMap<u64, HashSet<usize>>,
    archetypes: Vec<ArchetypeUnsafeFfi>,
}

impl ArchetypesHolder {
    pub fn new() -> Self {
        Self {
            archetype_id_by_components: HashMap::new(),
            archetype_ids_by_component: HashMap::new(),
            archetypes: Vec::new(),
        }
    }

    pub fn all(&self) -> &[ArchetypeUnsafeFfi] {
        &self.archetypes
    }

    pub fn by_id_ref(&self, id: usize) -> Option<&ArchetypeUnsafeFfi> {
        self.archetypes.get(id)
    }
    pub fn by_id_mut(&mut self, id: usize) -> Option<&mut ArchetypeUnsafeFfi> {
        self.archetypes.get_mut(id)
    }
    pub fn by_2_ids_ref(&self, mut id: (usize, usize)) -> Option<(&ArchetypeUnsafeFfi, &ArchetypeUnsafeFfi)> {
        if id.0 >= self.archetypes.len() || id.1 >= self.archetypes.len() {
            return None;
        }

        let should_swap = id.0 > id.1;

        if should_swap {
            id = (id.1, id.0);
        }

        let mut slices = self.archetypes.as_slice()[id.0..].split_at(id.1 - id.0);

        if should_swap {
            (slices.0, slices.1) = (slices.1, slices.0);
        }
        
        Some((&slices.0[0], &slices.1[0]))
    }
    pub fn by_2_ids_mut(&mut self, mut id: (usize, usize)) -> Option<(&mut ArchetypeUnsafeFfi, &mut ArchetypeUnsafeFfi)> {
        if id.0 >= self.archetypes.len() || id.1 >= self.archetypes.len() {
            return None;
        }

        let should_swap = id.0 > id.1;

        if should_swap {
            id = (id.1, id.0);
        }

        let mut slices = self.archetypes.as_mut_slice()[id.0..].split_at_mut(id.1 - id.0);

        if should_swap {
            (slices.0, slices.1) = (slices.1, slices.0);
        }

        Some((&mut slices.0[0], &mut slices.1[0]))
    }
    pub fn by_components_ref(&self, components: &UniqueComponentsSet) -> Option<&ArchetypeUnsafeFfi> {
        let id = self.id_by_components(components)?;
        self.by_id_ref(id)
    }
    pub fn by_components_mut(&mut self, components: &UniqueComponentsSet) -> Option<&mut ArchetypeUnsafeFfi> {
        let id = self.id_by_components(components)?;
        self.by_id_mut(id)
    }
    pub fn id_by_components(&self, components: &UniqueComponentsSet) -> Option<usize> {
        self.archetype_id_by_components.get(components).copied()
    }
    pub fn ids_by_component(&self, component: u64) -> Option<&HashSet<usize>> {
        self.archetype_ids_by_component.get(&component)
    }
    pub fn create(&mut self, components: UniqueComponentsSet) -> Result<usize, UniqueComponentsSet> {
        if self.archetype_id_by_components.contains_key(&components) {
            return Err(components);
        }

        let id = self.archetype_id_by_components.len();

        for component_type in components.components() {
            self.archetype_ids_by_component.entry(component_type.id).or_default().insert(id);
        }

        self.archetypes.push(ArchetypeUnsafeFfi::new_from_components(components.clone()));
        self.archetype_id_by_components.insert(components, id);

        Ok(id)
    }
    pub fn id_by_components_or_create(&mut self, components: UniqueComponentsSet) -> (usize, Option<UniqueComponentsSet>) {
        let Some(id) = self.id_by_components(&components) else {
            return (self.create(components).ok().unwrap(), None);
        };

        (id, Some(components))
    }
}