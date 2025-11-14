use fruits_ecs::WorldDataUnsafeFfi;

fn main() {
    println!("Hello, world!");
}

pub struct Registry {}

impl Registry {
    pub fn instantiate(prefab: &str, world: &mut WorldDataUnsafeFfi) {
        // todo: need to index assets in the project (use guid as reference to the material/texture/mesh)
    }
}
