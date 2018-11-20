#[macro_use]
extern crate nom;

use std::option;
use std::collections::HashMap;

#[derive(Debug,PartialEq)]
pub struct StructuredListItem<'a> {
  pub name: &'a str,
  pub kv: HashMap<&'a str, &'a str>
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
      kv:   map!(many0!(do_parse!(tag!("  ") >>
              val: separated_pair!(take_till!(|ch| ch == ':' || ch == '\r'), tag!(":"), take_till!(|ch| ch == ':' || ch == '\r')) >> (val))), |vec: Vec<_>| vec.into_iter().collect()) >>
            (StructuredListItem { name: name, kv: kv })
    )) |
    many0!(do_parse!(
            tag!("* ")  >>
      name: take_till!(|ch| ch == '\r') >>
            tag!("\r\n") >>
      kv:   map!(many0!(do_parse!(tag!("  ") >>
              val: separated_pair!(take_till!(|ch| ch == ':' || ch == '\r'), tag!(":"), take_till!(|ch| ch == ':' || ch == '\r')) >> (val))), |vec: Vec<_>| vec.into_iter().collect()) >>
            (StructuredListItem { name: name, kv: kv })
    ))
  )
);

fn main() {
    println!("Hello, world!");
}
