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

// TODO: Actually do something proper here, eg using bodil's typed_html or at least using string escaping
fn kv_to_html<'a>(kv: HashMap<&'a str, &'a str>) -> String {
  let mut acc = String::new();
  for (k, v) in kv.iter() {
    acc.push_str(&format!("{}{}{}{}", k, ": ", v, "<br/>\r"));
  }
  acc
}
fn ul_to_html(ul: Vec<StructuredListItem>) -> String {
  let mut acc = String::from("<ul>\r");
  for u in ul {
    acc.push_str(&format!("{}{}{}{}{}", "<li>", u.name, "<br/>", kv_to_html(u.kv.clone()), "</li>"));
  }
  acc.push_str("</ul>\r");
  acc
}
fn ol_to_html(ol: Vec<StructuredListItem>) -> String {
  let mut acc = String::from("<ol>\r");
  for o in ol {
    acc.push_str(&format!("{}{}{}{}{}", "<li>", o.name, "<br/>", kv_to_html(o.kv.clone()), "</li>"));
  }
  acc.push_str("</ol>\r");
  acc
}
fn all_to_html(node: &StructuredCollection) -> String {
  let mut acc = String::new();
  if node.level == 0 {
    acc.push_str(&format!("<html>\r<head>\r<title>{}</title>\r</head>\r<body>\r", node.name));
  } else {
    acc.push_str(&format!("<h{}>{}</h{}>\r", node.level, node.name, node.level));
  }
  for txt in node.text {
    acc.push_str(&format!("<p>{}</p>\r", txt));
  }
  acc.push_str(&ol_to_html(node.ol.clone()));
  acc.push_str(&ul_to_html(node.ul.clone()));
  for h in node.headings.iter() {
    acc.push_str(&all_to_html(h));
  }
  if node.level == 0 {
    acc.push_str("</body>\r</html>");
  }
  acc
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
  assert_eq!(list_items, sample_list_items);
  // HashMaps are unordered, so we can't do the naive check here, as it will sometimes give different serialisations
  //assert_eq!(ul_to_html(list_items), "<ul>\r<li>Foo bar baz<br/>key: value<br/>\r</li><li>Other stuff<br/>Description: Thing<br/>\rCaveat: Stuff<br/>\r</li><li>No kvs<br/></li></ul>\r");
}
