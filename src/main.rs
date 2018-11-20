#[macro_use]
extern crate nom;

use std::option;
use std::collections::HashMap;
use nom::IResult;

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

fn ul_wrapper<'a>(input: &'a str, sep: &'static str) -> IResult<&'a str, Vec<StructuredListItem<'a>>> {
    many0!(input, do_parse!(
            tag!(sep)  >>
      name: take_till!(|ch| ch == '\r') >>
            tag!("\r\n") >>
      kv:   map!(many0!(do_parse!(tag!("  ") >>
              val: separated_pair!(take_till!(|ch| ch == ':' || ch == '\r'), tag!(":"), take_till!(|ch| ch == ':' || ch == '\r')) >> (val))), |vec: Vec<_>| vec.into_iter().collect()) >>
            (StructuredListItem { name: name, kv: kv })
    ))
}

named!(ul<&str, Vec<StructuredListItem>>,
  alt!(
    apply!(ul_wrapper, "- ") | apply!(ul_wrapper, "* ")
  )
);

fn main() {
    println!("Hello, world!");
}
