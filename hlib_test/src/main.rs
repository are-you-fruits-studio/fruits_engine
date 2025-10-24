use std::collections::HashMap;

use fruits_serialization::*;

fn main() {
    let json = Person::<i32>::default().to_json();
    dbg!(&json);

    print!("\n>>>\n");

    let person = Person::<f64>::from_json(&json);
    dbg!(&person);

    print!("\n>>>\n");

    let mut person = Person::<i32>::default();
    let json = JsonValue::parse(&mut r#"{ "name": "hui", "short": { "2": "Bad" } }"#.chars()).unwrap();
    person.fill_partially_from_json(&json);
    dbg!(&person);
}

#[derive(JsonSerializable, Default, Debug)]
struct Person<T: JsonSerializable> {
    name: String,
    age: u32,
    feature: T,
    data: (),
    short: ShortPerson,
    costs: HashMap<SomeType, usize>,
    default_ty: SomeType,
}

#[derive(JsonSerializable, Debug, Default)]
struct ShortPerson(String, u32, SomeType);

#[derive(JsonSerializable, Debug, Default, PartialEq, Eq, Hash)]
enum SomeType {
    #[default]
    Good,
    Bad,
}
