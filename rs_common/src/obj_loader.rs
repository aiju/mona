#![allow(dead_code)]

use std::{
    io::{BufRead, BufReader, Lines},
    iter::Peekable,
    path::Path,
    str::Chars,
};

use crate::{
    geometry::{Vec2, Vec3},
    mesh::{self, Mesh},
};

struct ItemParser<'a> {
    iter: Peekable<Chars<'a>>,
}

#[derive(Debug)]
enum Item {
    Vertex([f64; 3], Option<f64>),
    VertexNormal([f64; 3]),
    VertexTexture(Vec<f64>),
    Face(Vec<(isize, Option<isize>, Option<isize>)>),
    UseMtl(String),
    MtlLib(String),
}

#[derive(Debug)]
enum MtlItem {
    NewMtl(String),
    MapKd(String),
}

impl<'a> ItemParser<'a> {
    fn new(str: &'a str) -> Self {
        Self {
            iter: str.chars().peekable(),
        }
    }
    fn skip_whitespace(&mut self) {
        while self.iter.next_if(|c| c.is_ascii_whitespace()).is_some() {}
    }
    fn opt_field(&mut self) -> Option<String> {
        self.skip_whitespace();
        let mut s = String::new();
        while let Some(c) = self.iter.next_if(|&c| !c.is_ascii_whitespace() && c != '#') {
            s.push(c);
        }
        if s.len() > 0 { Some(s) } else { None }
    }
    fn field(&mut self) -> String {
        self.opt_field().expect("expected field")
    }
    fn opt_slashed_field(&mut self) -> Option<String> {
        let mut s = String::new();
        while let Some(c) = self
            .iter
            .next_if(|&c| !c.is_ascii_whitespace() && c != '#' && c != '/')
        {
            s.push(c);
        }
        if s.len() > 0 { Some(s) } else { None }
    }
    fn opt_float(&mut self) -> Option<f64> {
        self.opt_field()
            .map(|v| v.parse().expect(&format!("expected float, got \"{v}\"")))
    }
    fn opt_slashed_int(&mut self) -> Option<isize> {
        self.opt_slashed_field()
            .map(|v| v.parse().expect(&format!("expected float, got \"{v}\"")))
    }
    fn float(&mut self) -> f64 {
        self.opt_float().expect("expected float")
    }
    fn eol(&mut self) {
        while self.iter.next_if(|&c| c.is_ascii_whitespace()).is_some() {}
        if let Some('#') | None = self.iter.peek() {
        } else {
            let s: String = self
                .iter
                .by_ref()
                .take_while(|c| *c != '\n' && *c != '#')
                .collect();
            panic!("obj parser: unexpected data on end of line: \"{s}\"");
        }
    }
    fn vertex(&mut self) -> Item {
        let x = self.float();
        let y = self.float();
        let z = self.float();
        let w = self.opt_float();
        self.eol();
        Item::Vertex([x, y, z], w)
    }
    fn vertex_normal(&mut self) -> Item {
        let x = self.float();
        let y = self.float();
        let z = self.float();
        self.eol();
        Item::VertexNormal([x, y, z])
    }
    fn vertex_texture(&mut self) -> Item {
        let mut vec = Vec::new();
        vec.push(self.float());
        self.opt_float().into_iter().for_each(|x| vec.push(x));
        self.opt_float().into_iter().for_each(|x| vec.push(x));
        self.eol();
        Item::VertexTexture(vec)
    }
    fn face(&mut self) -> Item {
        let mut vec = Vec::new();
        self.skip_whitespace();
        while let Some(n) = self.opt_slashed_int() {
            let mut m = None;
            let mut k = None;
            if self.iter.next_if_eq(&'/').is_some() {
                m = self.opt_slashed_int();
                if self.iter.next_if_eq(&'/').is_some() {
                    k = self.opt_slashed_int();
                }
            }
            vec.push((n, m, k));
            self.skip_whitespace();
        }
        self.eol();
        Item::Face(vec)
    }
    fn usemtl(&mut self) -> Item {
        let name = self.field();
        self.eol();
        Item::UseMtl(name)
    }
    fn mtllib(&mut self) -> Item {
        let name = self.field();
        self.eol();
        Item::MtlLib(name)
    }
    fn newmtl(&mut self) -> MtlItem {
        let name = self.field();
        self.eol();
        MtlItem::NewMtl(name)
    }
    fn map_kd(&mut self) -> MtlItem {
        let name = self.field();
        self.eol();
        MtlItem::MapKd(name)
    }
    fn parse(&mut self) -> Option<Item> {
        if let Some(ty) = self.opt_field() {
            Some(match ty.as_str() {
                "v" => self.vertex(),
                "vn" => self.vertex_normal(),
                "vt" => self.vertex_texture(),
                "f" => self.face(),
                "usemtl" => self.usemtl(),
                "mtllib" => self.mtllib(),
                _ => {
                    eprint!("obj parser: skipping unknown item \"{ty}\"\n");
                    return None;
                }
            })
        } else {
            self.eol();
            return None;
        }
    }
    fn mtl_parse(&mut self) -> Option<MtlItem> {
        if let Some(ty) = self.opt_field() {
            Some(match ty.as_str() {
                "newmtl" => self.newmtl(),
                "map_Kd" => self.map_kd(),
                _ => {
                    eprint!("obj parser: skipping unknown mtl item \"{ty}\"\n");
                    return None;
                }
            })
        } else {
            self.eol();
            return None;
        }
    }
}

pub struct Material {
    name: String,
    texture: Option<String>,
}

#[derive(Default)]
pub struct MtlLoader {
    materials: Vec<Material>,
}

impl MtlLoader {
    pub fn parse(mut self, obj_path: &str, path: &str) -> Vec<Material> {
        let p = Path::new(obj_path).parent().unwrap().join(path);
        let f = BufReader::new(std::fs::File::open(p).unwrap());
        for line in f.lines() {
            match ItemParser::new(&line.unwrap()).mtl_parse() {
                Some(MtlItem::NewMtl(name)) => self.materials.push(Material {
                    name,
                    texture: None,
                }),
                Some(MtlItem::MapKd(name)) => {
                    self.materials.last_mut().unwrap().texture = Some(name)
                }
                None => {}
            }
        }
        self.materials
    }
}

pub struct ObjLoader {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    uv: Vec<Vec2>,
    mesh: Mesh,
    materials: Vec<Material>,
    current_material: usize,
}

impl ObjLoader {
    pub fn new() -> Self {
        let mut mesh = Mesh::default();
        mesh.triangles.push(vec![]);
        mesh.materials.push(mesh::Material { texture: None });
        ObjLoader {
            vertices: Vec::new(),
            normals: Vec::new(),
            uv: Vec::new(),
            mesh,
            materials: vec![Material {
                name: "default".into(),
                texture: None,
            }],
            current_material: 0,
        }
    }
    fn lookup_vertex(&mut self, n: isize) -> Vec3 {
        assert!(n != 0);
        if n > 0 {
            return self.vertices[(n - 1) as usize];
        } else {
            return self.vertices[self.vertices.len() - (-n) as usize];
        }
    }
    fn lookup_normal(&mut self, n: isize) -> Vec3 {
        assert!(n != 0);
        if n > 0 {
            return self.normals[(n - 1) as usize];
        } else {
            return self.normals[self.normals.len() - (-n) as usize];
        }
    }
    fn lookup_uv(&mut self, n: isize) -> Vec2 {
        assert!(n != 0);
        if n > 0 {
            return self.uv[(n - 1) as usize];
        } else {
            return self.uv[self.uv.len() - (-n) as usize];
        }
    }
    fn lookup_texture(&mut self, name: &str) -> usize {
        self.mesh
            .textures
            .iter()
            .enumerate()
            .find_map(|(i, n)| (n == name).then_some(i))
            .unwrap_or_else(|| {
                let i = self.mesh.textures.len();
                self.mesh.textures.push(name.to_string());
                i
            })
    }
    fn process_triangle(&mut self, vert: [(isize, Option<isize>, Option<isize>); 3]) {
        let v = vert.map(|(v, _, _)| self.lookup_vertex(v).into());
        let t = vert.map(|(_, t, _)| t.map(|t| self.lookup_uv(t)).unwrap_or_default());
        self.mesh.triangles[self.current_material].push(mesh::Triangle {
            vertices: v,
            uv: t,
            rgb: [!0; 3],
        });
    }
    pub fn process_face(&mut self, face: Vec<(isize, Option<isize>, Option<isize>)>) {
        assert!(face.len() == 3 || face.len() == 4);
        if face.len() == 3 {
            self.process_triangle([face[0], face[1], face[2]]);
        } else {
            self.process_triangle([face[0], face[1], face[2]]);
            self.process_triangle([face[0], face[2], face[3]]);
        }
    }
    pub fn parse<T: BufRead>(mut self, path: &str, lines: Lines<T>) -> Mesh {
        for line in lines {
            match ItemParser::new(&line.unwrap()).parse() {
                Some(Item::Vertex(v, _)) => self.vertices.push(v.into()),
                Some(Item::VertexNormal(v)) => self.normals.push(v.into()),
                Some(Item::VertexTexture(v)) => self.uv.push([v[0], v[1]].into()),
                Some(Item::Face(face)) => self.process_face(face),
                Some(Item::MtlLib(mtl_path)) => {
                    for mtl in MtlLoader::default().parse(path, &mtl_path) {
                        let texture = mtl.texture.as_ref().map(|name| self.lookup_texture(&name));
                        self.materials.push(mtl);
                        self.mesh.triangles.push(Vec::new());
                        self.mesh.materials.push(mesh::Material { texture });
                    }
                }
                Some(Item::UseMtl(material)) => {
                    if let Some((idx, _)) = self
                        .materials
                        .iter()
                        .enumerate()
                        .find(|(_, m)| m.name == material)
                    {
                        self.current_material = idx;
                    } else {
                        eprintln!("missing material \"{material}\"");
                    }
                }
                None => {}
            }
        }
        self.mesh
    }
}

pub fn load_obj_file(path: &str) -> Mesh {
    let f = BufReader::new(std::fs::File::open(path).unwrap());
    ObjLoader::new().parse(path, f.lines())
}
