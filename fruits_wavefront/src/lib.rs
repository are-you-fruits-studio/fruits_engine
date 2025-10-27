#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ObjVertex {
    pub v: usize,
    pub vt: Option<usize>,
    pub vn: Option<usize>,
}

#[derive(Debug, Default)]
pub struct ObjMesh {
    pub positions: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub normals: Vec<[f32; 3]>,
    pub faces: Vec<[ObjVertex; 3]>,
}

pub fn parse_obj(raw: &str) -> Option<ObjMesh> {
    let mut mesh = ObjMesh::default();

    for line in raw.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let mut parts = line.split_whitespace();
        let Some(tag) = parts.next() else { continue };

        match tag {
            "v" => {
                let mut parts = parts.map(|s| s.parse().unwrap());
                mesh.positions.push([parts.next()?, parts.next()?, parts.next()?]);
            }
            "vt" => {
                let mut parts = parts.map(|s| s.parse().unwrap());
                mesh.texcoords.push([parts.next()?, parts.next()?]);
            }
            "vn" => {
                let mut parts = parts.map(|s| s.parse().unwrap());
                mesh.normals.push([parts.next()?, parts.next()?, parts.next()?]);
            }
            "f" => {
                let mut face = Vec::new();
                for part in parts {
                    // e.g., "3/4/5" or "3//5" or "3"
                    let mut indices = part.split('/');
                    let v = parse_index(indices.next()?, mesh.positions.len());
                    let vt = indices.next().and_then(|s| if !s.is_empty() { Some(parse_index(s, mesh.texcoords.len())) } else { None });
                    let vn = indices.next().and_then(|s| if !s.is_empty() { Some(parse_index(s, mesh.normals.len())) } else { None });
                    face.push(ObjVertex { v, vt, vn });
                }

                // Triangulate (OBJ can have quads)
                for tri in face.windows(3) {
                    mesh.faces.push([tri[0], tri[1], tri[2]]);
                }
            }
            _ => {} // ignore others for now
        }
    }

    Some(mesh)
}

fn parse_index(idx: &str, len: usize) -> usize {
    let i: isize = idx.parse().unwrap();

    if i < 0 {
        (len as isize + i) as usize
    } else {
        (i as usize) - 1 // OBJ is 1-based
    }
}