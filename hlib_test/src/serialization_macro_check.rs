use fruits_engine::*;

#[derive(TransSerializable)]
pub struct SomeStruct<'a, 'b, T>
    where T : Copy
{
    name: &'b String,
    age: &'a u32,
    data: T,
    unit: SomeUnit,
}

#[derive(TransSerializable)]
pub struct SomeTuple(String, u32);

#[derive(TransSerializable)]
pub struct SomeUnit;

#[derive(TransSerializable)]
pub enum SomeEnum {
    A,
    B(u32),
    C { name: String },
    D(String, String, u32, Option<u32>),
    E { name1: String, password: String, age: Option<u8> }
}
