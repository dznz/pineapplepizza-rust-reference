#[macro_use]
extern crate nom;

use std::option;

#[derive(Debug,PartialEq)]
pub struct StructuredListItem<'a> {
  name: &'a str
}

#[derive(Debug,PartialEq)]
pub struct StructuredCollection<'a> {
  level: u8,
  name: &'a str,
  text: Option<&'a str>,
  ol: Vec<StructuredListItem<'a>>,
  ul: Vec<StructuredListItem<'a>>,
  headings: Vec<StructuredCollection<'a>>
}

fn main() {
    println!("Hello, world!");
}
