#[macro_use]
extern crate nom;

use std::option;
use std::collections::HashMap;
use nom::IResult;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;


#[derive(Debug,PartialEq,Eq,Clone)]
pub struct StructuredListItem<'a> {
  pub name: &'a str,
  pub kv: HashMap<&'a str, &'a str>
}

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct StructuredCollection<'a> {
  level: u8,
  name: &'a str,
  text: Option<&'a str>,
  ol: Vec<StructuredListItem<'a>>,
  ul: Vec<StructuredListItem<'a>>,
  headings: Vec<StructuredCollection<'a>>
}

fn ul_wrapper<'a>(input: &'a str, sepb: bool) -> IResult<&'a str, Vec<StructuredListItem<'a>>> {
    let sep = (if sepb {"- "} else {"* "});
    many1!(input, do_parse!(
            tag!(sep)  >>
      name: take_till!(|ch| ch == '\n') >>
            tag!("\n") >>
      kv:   map!(many0!(do_parse!(tag!("  ") >>
              val: separated_pair!(take_till!(|ch| ch == ':' || ch == '\n'), tag!(": "), take_till!(|ch| ch == ':' || ch == '\n')) >> tag!("\n") >> (val))), |vec: Vec<_>| vec.into_iter().collect()) >>
            (StructuredListItem { name: name, kv: kv })
    ))
}

named!(ul<&str, Vec<StructuredListItem>>,
  alt!(
    apply!(ul_wrapper, false) | apply!(ul_wrapper, true)
  )
);

fn main() {
    println!("Hello, world!");
}

#[test]
fn parse_list_items() {
  let sample_list_items = vec!(
    StructuredListItem {
      name: "Foo bar baz",
      kv: [("key", "value")].iter().cloned().collect(),
    },
    StructuredListItem {
      name: "Other stuff",
      kv: [("Description", "Thing"), ("Caveat", "Stuff")].iter().cloned().collect(),
    },
    StructuredListItem {
      name: "No kvs",
      kv: [].iter().cloned().collect(),
    },
  );
  let path = Path::new("list_items.🍍🍕.slice");
  let display = path.display();

  // Open the path in read-only mode, returns `io::Result<File>`
  let mut file = match File::open(&path) {
      // The `description` method of `io::Error` returns a string that
      // describes the error
      Err(why) => panic!("couldn't open {}: {}", display,
                                                 why.description()),
      Ok(file) => file,
  };

  // Read the file contents into a string, returns `io::Result<usize>`
  let mut s = String::new();
  match file.read_to_string(&mut s) {
      Err(why) => panic!("couldn't read {}: {}", display,
                                                 why.description()),
      Ok(_) => print!("{} contains:\n{}", display, s),
  }
  let foo = s.clone();
  let list_items_maybe = ul(&foo);
  let list_items = match list_items_maybe {
    Err(why) => panic!("{}", why.description()),
    Ok((_, x))    => x.clone()
  };
  assert_eq!(list_items, sample_list_items)
  //print!("{:?}", list_items_maybe); //assert_eq!(list_items, sample_list_items)
    
}
