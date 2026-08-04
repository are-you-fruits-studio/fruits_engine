use std::ops::{Deref, DerefMut};

use fruits_ecs::Resource;

use crate::{GlobalSerializer, StandardTransSerializer};

#[repr(C)]
#[derive(Default)]
pub struct SerializersResource(pub GlobalSerializer);

impl Resource for SerializersResource { }

impl Deref for SerializersResource {
    type Target = GlobalSerializer;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerializersResource {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}