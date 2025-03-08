#![allow(dead_code)]

use std::{
    io::{BufRead, BufReader, Lines},
    iter::Peekable,
    str::Chars,
};

use crate::BarePrimitive;

struct ItemParser<'a> {
    iter: Peekable<Chars<'a>>,
}

#[derive(Debug)]
enum Item {
    Vertex([f64; 3], Option<f64>),
    VertexNormal([f64; 3]),
    VertexTexture(Vec<f64>),
    Face(Vec<(isize, Option<isize>, Option<isize>)>),
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
    fn parse(&mut self) -> Option<Item> {
        if let Some(ty) = self.opt_field() {
            Some(match ty.as_str() {
                "v" => self.vertex(),
                "vn" => self.vertex_normal(),
                "vt" => self.vertex_texture(),
                "f" => self.face(),
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
}

pub struct ObjLoader {
    vertices: Vec<[f64; 3]>,
    normals: Vec<[f64; 3]>,
    uv: Vec<[f64; 2]>,
    triangles: Vec<BarePrimitive>,
}

impl ObjLoader {
    pub fn new() -> Self {
        ObjLoader {
            vertices: Vec::new(),
            normals: Vec::new(),
            uv: Vec::new(),
            triangles: Vec::new(),
        }
    }
    fn lookup_vertex(&mut self, n: isize) -> [f64; 3] {
        assert!(n != 0);
        if n > 0 {
            return self.vertices[(n - 1) as usize];
        } else {
            return self.vertices[self.vertices.len() - (-n) as usize];
        }
    }
    fn lookup_normal(&mut self, n: isize) -> [f64; 3] {
        assert!(n != 0);
        if n > 0 {
            return self.normals[(n - 1) as usize];
        } else {
            return self.normals[self.normals.len() - (-n) as usize];
        }
    }
    fn lookup_uv(&mut self, n: isize) -> [f64; 2] {
        assert!(n != 0);
        if n > 0 {
            return self.uv[(n - 1) as usize];
        } else {
            return self.uv[self.uv.len() - (-n) as usize];
        }
    }
    fn process_triangle(&mut self, vert: [(isize, Option<isize>, Option<isize>); 3]) {
        let v = vert.map(|(v, _, _)| {
            let mut c = [0.0, 0.0, 0.0, 1.0];
            c[0..3].copy_from_slice(&self.lookup_vertex(v));
            c
        });
        let t = vert.map(|(_, t, _)| t.map(|t| self.lookup_uv(t)).unwrap_or([0.0, 0.0]));
        self.triangles.push(BarePrimitive { vertices: v, uv: t, rgb: [!0; 3] });
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
    pub fn parse<T: BufRead>(mut self, lines: Lines<T>) -> Vec<BarePrimitive> {
        for line in lines {
            match ItemParser::new(&line.unwrap()).parse() {
                Some(Item::Vertex(v, _)) => self.vertices.push(v),
                Some(Item::VertexNormal(v)) => self.normals.push(v),
                Some(Item::VertexTexture(v)) => self.uv.push(v.try_into().unwrap()),
                Some(Item::Face(face)) => self.process_face(face),
                None => {}
            }
        }
        self.triangles
    }
}

pub fn load_obj_file(path: &str) -> Vec<BarePrimitive> {
    let f = BufReader::new(std::fs::File::open(path).unwrap());
    ObjLoader::new().parse(f.lines())
}

#[test]
fn foo() {
    let f = BufReader::new(std::fs::File::open("/home/aiju/untitled.obj").unwrap());
    let tris = ObjLoader::new().parse(f.lines());
    println!("{tris:?}");
}
