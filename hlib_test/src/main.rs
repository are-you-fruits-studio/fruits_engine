use fruits_serialization::*;

fn main() {
    const RAW_DATA: &str = "
v 0.123 0.234 0.345 1.0

vt 0.500 1 [0]

vn 0.707 0.000 0.707

vp 0.310000 3.210000 2.100000

# Polygonal face element (see below)
f 1 2 3
f 3/1 4/2 5/3
f 6/4/1 3/5/3 7/6/5
f 7//1 8//2 9//3
    ";

    let obj = fruits_wavefront::parse_obj(RAW_DATA);

    dbg!(obj);
}

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