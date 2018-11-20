#[macro_use]
extern crate nom;

use std::option;

#[derive(Debug,PartialEq)]
pub struct StructuredListItem<'a> {
  pub name: &'a str
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

named!(ul<&str, Vec<StructuredListItem>>,
  alt!(
    many0!(do_parse!(
            tag!("- ")  >>
      name: take_till!(|ch| ch == '\r') >>
            tag!("\r\n") >>
            (StructuredListItem { name: name })
    )) |
    many0!(do_parse!(
            tag!("* ")  >>
      name: take_till!(|ch| ch == '\r') >>
            tag!("\r\n") >>
            (StructuredListItem { name: name })
    ))
  )
);

fn main() {
    println!("Hello, world!");
}
