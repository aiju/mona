#![allow(dead_code)]

use std::{iter::Peekable, str::Chars};

struct Parser<'a> {
    lineno: usize,
    iter: Peekable<Chars<'a>>,
}

#[derive(Debug)]
enum Item {
    Vertex([f64; 3]),
}

impl<'a> Parser<'a> {
    fn skip_comments(&mut self) {
        while let Some(c) = self.iter.peek() {
            match *c {
                '#' => while self.iter.next_if(|c| *c != '\n').is_some() {},
                '\n' => {
                    self.iter.next();
                    self.lineno += 1;
                }
                c if c.is_ascii_whitespace() => {
                    self.iter.next();
                }
                _ => break,
            }
        }
    }
    fn skip_whitespace(&mut self) {
        while self
            .iter
            .next_if(|c| *c != '\n' && c.is_ascii_whitespace())
            .is_some()
        {}
    }
    fn opt_field(&mut self) -> Option<String> {
        self.skip_whitespace();
        let s: String = self
            .iter
            .by_ref()
            .take_while(|c| !c.is_ascii_whitespace() && *c != '#')
            .collect();
        if s.len() > 0 { Some(s) } else { None }
    }
    fn field(&mut self) -> String {
        self.opt_field().expect("expected field")
    }
    fn float(&mut self) -> f64 {
        self.field().parse().expect("expected float")
    }
    fn vertex(&mut self) -> Item {
        let x = self.float();
        let y = self.float();
        let z = self.float();
        Item::Vertex([x, y, z])
    }
    fn line(&mut self) -> Option<Item> {
        self.skip_comments();
        self.opt_field().map(|field| match field.as_str() {
            "v" => self.vertex(),
            _ => panic!("unexpected line {field}"),
        })
    }
}

#[test]
fn foo() {
    let s = String::from_utf8(std::fs::read("/home/aiju/untitled.obj").unwrap()).unwrap();
    let mut parser = Parser {
        lineno: 1,
        iter: s.chars().peekable()
    };
    while let Some(item) = parser.line() {
        println!("{:?}", item);
    }
}
