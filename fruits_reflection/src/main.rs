use fruits_reflection::*;

fn main() {
    let person_type = Person::get_type();

    println!("{:?}", person_type.create_instance_any(vec![Box::new(String::from("Serhii")), Box::new(5_u8)]).downcast::<Person>().unwrap());
    println!("{:?}", person_type.create_instance::<Person>(vec![Box::new(String::from("Dmytro")), Box::new(35_u8)]).unwrap());
}

#[derive(Debug)]
pub struct Person {
    pub name: String,
    pub age: u8,
}

//

impl Person {
    pub fn get_type() -> ReflType {
        ReflType::new(|(name, age)| {
            Person {
                name,
                age,
            }
        })
    }
}
