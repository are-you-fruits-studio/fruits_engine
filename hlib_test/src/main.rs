use std::collections::HashMap;

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