#[macro_use]
extern crate nom;

extern crate unicode_segmentation;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use std::env;
use std::option;
use std::collections::HashMap;
use nom::{IResult,InputTakeAtPosition,AsChar,ErrorKind};

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use nom::ErrorKind::Custom;
use serde::ser::{Serialize, Serializer, SerializeSeq, SerializeMap};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug,PartialEq,Eq,Clone,Serialize)]
pub struct StructuredListItem<'a> {
  pub name: &'a str,
  #[serde(skip_serializing_if = "HashMap::is_empty")]
  pub kv: HashMap<&'a str, &'a str>
}

#[derive(Debug,PartialEq,Eq,Clone)]
pub struct StructuredOrderedListItem<'a> {
  pub name: &'a str,
}

#[derive(Debug,PartialEq,Eq,Clone,Serialize)]
pub struct StructuredCollection<'a> {
  #[serde(skip_serializing)]
  level: u8,
  name: &'a str,
  #[serde(skip_serializing_if = "Option::is_none")]
  text: Option<String>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  ol: Vec<StructuredOrderedListItem<'a>>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  ul: Vec<StructuredListItem<'a>>,
  #[serde(skip_serializing_if = "Vec::is_empty")]
  headings: Vec<StructuredCollection<'a>>
}

impl<'a> Serialize for StructuredOrderedListItem<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.name)
    }
}

// TODO: Actually do something proper here, eg using bodil's typed_html or at least using string escaping
fn kv_to_html<'a>(kv: HashMap<&'a str, &'a str>) -> String {
  let mut acc = String::new();
  for (k, v) in kv.iter() {
    acc.push_str(&format!("{}{}{}{}", k, ": ", v, "<br/>\n"));
  }
  acc
}
fn ul_to_html(ul: Vec<StructuredListItem>) -> String {
  if ul.len() == 0 {
    return String::new();
  }
  let mut acc = String::from("<ul>\n");
  for u in ul {
    acc.push_str(&format!("{}{}{}{}{}", "<li>", u.name, "<br/>", kv_to_html(u.kv.clone()), "</li>"));
  }
  acc.push_str("</ul>\n");
  acc
}
fn ol_to_html(ol: Vec<StructuredOrderedListItem>) -> String {
  if ol.len() == 0 {
    return String::new();
  }
  let mut acc = String::from("<ol>\n");
  for o in ol {
    acc.push_str(&format!("{}{}{}", "<li>", o.name, "</li>"));
  }
  acc.push_str("</ol>\n");
  acc
}
fn all_to_html(node: &StructuredCollection) -> String {
  let mut acc = String::new();
  if node.level == 0 {
    let foo = format!("<html>\n<head>\n<title>{}</title>\n</head>\n<body>\n", node.name);
    acc.push_str(&foo);
  } else {
    let foo = format!("<h{}>{}</h{}>\n", node.level, node.name, node.level);
    acc.push_str(&foo);
  }
  for txt in node.text.clone() {
    acc.push_str(&format!("<p>{}</p>\n", txt));
  }
  acc.push_str(&ol_to_html(node.ol.clone()));
  acc.push_str(&ul_to_html(node.ul.clone()));
  for h in node.headings.iter() {
    acc.push_str(&all_to_html(h));
  }
  if node.level == 0 {
    acc.push_str("</body>\n</html>");
  }
  acc
}

fn ul_wrapper<'a>(input: &'a str, sepb: bool) -> IResult<&'a str, Vec<StructuredListItem<'a>>> {
    let sep = (if sepb {"- "} else {"* "});
    do_parse!(input,
      it: many1!(do_parse!(
              tag!(sep)  >>
        name: take_till!(|ch| ch == '\n') >>
              tag!("\n") >>
        kv:   map!(many0!(do_parse!(tag!("  ") >>
                val: separated_pair!(take_till!(|ch| ch == ':' || ch == '\n'), tag!(": "), take_till!(|ch| ch == ':' || ch == '\n')) >> tag!("\n") >> (val))), |vec: Vec<_>| vec.into_iter().collect()) >>
              (StructuredListItem { name: name, kv: kv })
      )) >> tag!("\n") >> (it)
    )
}

fn ol_wrapper<'a>(input: &'a str) -> IResult<&'a str, Vec<StructuredOrderedListItem<'a>>> {
    do_parse!(input,
      it: many1!(do_parse!(
        num:  call!(nom::digit) >> // Eventually, validate using this
              tag!(". ")  >>
        name: take_till!(|ch| ch == '\n') >>
              tag!("\n") >>
              (StructuredOrderedListItem { name: name })
      )) >> tag!("\n") >> (it)
    )
}

named!(linecomment<&str, &str>,
  do_parse!(tag!("//") >> take_till!(|ch| ch == '\n') >> tag!("\n") >> (""))
);

named!(spancomment<&str, String>,
  do_parse!(tag!("/*") >> take_until_and_consume!("*/") >> ("".to_string()))
);

named!(ul<&str, Vec<StructuredListItem>>,
  alt!(
    apply!(ul_wrapper, false) | apply!(ul_wrapper, true) | value!(vec!())
  )
);

named!(ol<&str, Vec<StructuredOrderedListItem>>,
  alt!(
    call!(ol_wrapper) | value!(vec!())
  )
);

named!(take_till_eol_with_comments<&str, String>,
       complete!(do_parse!(txt: complete!(many1!(alt_complete!(
           escaped_transform!(call!(accept_stuff), '\\',
             alt_complete!(
                 tag!("n")  => {|_| "\n".to_string()}
               | tag!("\\") => {|_| "\\".to_string()}
               | tag!("/")  => {|_| "/".to_string()}
               | tag!("\n") => {|_| "".to_string()}
             )
         )
         | spancomment))) >> end: complete!(alt!(tag!("\n") | complete!(linecomment))) >> (txt.into_iter().collect())))
);

pub fn accept_stuff<T>(input: T) -> IResult<T, T, u32>
where
  T: InputTakeAtPosition,
  <T as InputTakeAtPosition>::Item: AsChar,
{
  input.split_at_position1({|item| { let ch = item.as_char(); (((ch == '\n') || (ch == '\\')) || (ch == '/'))} }, ErrorKind::Custom(2))
}

fn h_wrapper<'a>(input: &'a str, level: u8) -> IResult<&'a str, StructuredCollection> {
  if(level == 0) {
    let x = alt!(input,
              do_parse!(tag!("# ") >> name: take_till!(|ch| ch == '\n') >> tag!("\n\n") >> (name)) |
              map!(verify!(do_parse!(name: take_till!(|ch| ch == '\n') >> tag!("\n") >> underscore: take_while!(|c| c == '=') >> tag!("\n\n") >> (name, underscore)), |(txt, underscore): (&str, &str)| UnicodeSegmentation::graphemes(txt, true).collect::<Vec<&str>>().len() == UnicodeSegmentation::graphemes(underscore, true).collect::<Vec<&str>>().len()), |(txt, len)| txt));
    match x {
      Ok((rest, name)) => h_sub_wrapper(rest, level, name),
      _                => panic!("No header found.")
    }
  } else {
    let hashes: &str = &"#".repeat(level as usize + 1);
    let x = do_parse!(input, tag!(hashes) >> many0!(tag!(" ")) >> name: take_till!(|ch| ch == '\n') >> tag!("\n") >> (name));
    match x {
      Ok((rest, name)) => h_sub_wrapper(rest, level, name),
      _                => Err(nom::Err::Error(error_position!(input, nom::ErrorKind::Custom(1)))) // panic!("Do some actual checking here")
    }
  }
}
fn h_sub_wrapper<'a>(input: &'a str, level: u8, name: &'a str) -> IResult<&'a str, StructuredCollection<'a>> {
  do_parse!(input,
    // Assume that something is text until we hit a start token for anything else
    text: many0!(do_parse!(
            // Check against start tokens - TODO: be more thorough about this
            not!(alt!(eof!() | tag!("---\n") | tag!("- ") | tag!("* ") | do_parse!(call!(nom::digit) >> tag!(". ") >> ("")) | do_parse!(tag!("#") >> take_while!(|c| c == '#') >> tag!(" ") >> ("")))) >>
            line: call!(take_till_eol_with_comments) >> tag!("\n") >>
            (line)
            )) >>
    ols: call!(ol) >>
    uls: call!(ul) >>
    headings: many0!(apply!(h_wrapper, level + 1)) >>
    (StructuredCollection {
      level: level,
      name: name,
      text: (if text.len() == 0 || (text.len() == 1 && text[0] == "") { None } else { Some(text.iter().fold(String::new(), |acc, x| acc + &format!("{}\n", x)).clone()) }),
      ol: ols,
      ul: uls,
      headings: headings
    })
  )
}

named!(document<&str, StructuredCollection>,
    apply!(h_wrapper, 0)
);

fn main() -> std::io::Result<()> {
  let args: Vec<String> = env::args().collect();
  if (args.len() < 3) {
    println!("Usage: pineapplepizza [FILE] (--html|--rust-debug|--json) [OUTPUT?]");
    return Ok(());
  }
  let input_file = args[1].clone();
  let conversion_type_ = args[2].clone();
  let conversion_type = conversion_type_.as_str();
  let print_out = (args.len() == 3);
  let path_in = Path::new(&input_file);
  let mut file = match File::open(&path_in) {
    // The `description` method of `io::Error` returns a string that
    // describes the error
    Err(why) => panic!("couldn't open {}: {}", &input_file,
                                               why.description()),
    Ok(file) => file,
  };
  // Read the file contents into a string, returns `io::Result<usize>`
  let mut s = String::new();
  match file.read_to_string(&mut s) {
      Err(why) => panic!("couldn't read {}: {}", &input_file,
                                                 why.description()),
      _        => ()
  }
  let foo = s.clone();
  let doc = match document(&foo) {
    Ok((_, it)) => it,
    bad         => panic!(format!("{:?}", bad))
  };
  let output = match conversion_type {
    "--rust-debug" => format!("{:?}", doc),
    "--html"       => format!("{}", all_to_html(&doc)),
    "--json"       => match serde_json::to_string(&doc) {
      Ok(x)  => format!("{}", x),
      Err(x) => panic!("{}", x),
    },
    x              => panic!("Did not understand flag: {}", x),
  };
  if print_out {
    println!("{}", output);
    Ok(())
  } else {
    let output_file = args[3].clone();
    //let path_out = Path::new(&output_file);
    let mut out = File::create(output_file)?;
    write!(out, "{}", output)?;
    Ok(())
  }
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
  let path = Path::new("examples/list_items.🍍🍕.slice");
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

#[test]
fn parse_text_comments() {
  let test_set = vec!(("foo bar baz\n\n\\/\\/foo\nquuz\n\n", "foo bar baz"),("foo bar baz //comment\n\n\\/\\/foo\nquuz\n\n", "foo bar baz "),("foo \\/ \\\\baz\n\n\\/\\/foo\nquuz\n\n", "foo / \\baz")/*,("foo /*bar*/ baz\n\n\\/\\/foo\nquuz\n\n", "foo  baz")*/);
  for (test, expected) in test_set {
    let result = match take_till_eol_with_comments(&test) {
      Ok((_, x)) => x,
      y          => panic!("{:#?}", y)
    };
    assert_eq!(expected, result);
  }
}
/*
#[test]
fn parse_file_test() {
  let files = vec!(("examples/welp.🍍🍕", StructuredCollection { level: 0, name: "🍍🍕", text: Some("dklh\n\n".to_string()), ol: vec![], ul: vec![], headings: vec![StructuredCollection { level: 1, name: "1", text: None, ol: vec![], ul: vec![], headings: vec![] }, StructuredCollection { level: 1, name: "2", text: None, ol: vec![], ul: vec![], headings: vec![] }, StructuredCollection { level: 1, name: "3", text: None, ol: vec![], ul: vec![], headings: vec![StructuredCollection { level: 2, name: "4", text: None, ol: vec![], ul: vec![], headings: vec![] }, StructuredCollection { level: 2, name: "5", text: None, ol: vec![], ul: vec![], headings: vec![] }] }, StructuredCollection { level: 1, name: "6", text: Some("\nfwfd\n\niwdefwedfgwfgwd\n".to_string()), ol: vec![], ul: vec![], headings: vec![] }] }), ("examples/self.🍍🍕", StructuredCollection { level: 0, name: "🍍🍕", text: Some("Pineapplepizza example file.\n\n".to_string()), ol: vec![StructuredOrderedListItem { name: "We can list things." }, StructuredOrderedListItem { name: "One after the other." }], ul: vec![], headings: vec![StructuredCollection { level: 1, name: "We can have empty sections", text: Some("\nqwldjkhhf\n\n".to_string()), ol: vec![StructuredOrderedListItem { name: "wdlkfjhwqdfh" }, StructuredOrderedListItem { name: "lkdsahflkdfh" }], ul: vec![], headings: vec![StructuredCollection { level: 2, name: "Or sections with things in them", text: Some("\nwlfjkehwldfh\n\n".to_string()), ol: vec![StructuredOrderedListItem { name: "Such as ordered lists" }, StructuredOrderedListItem { name: "ghwelkhglkwerhg" }, StructuredOrderedListItem { name: "lkdjhflwdhflwdf" }], ul: vec![], headings: vec![] }, StructuredCollection { level: 2, name: "Sub sub heading", text: Some("\nStuff\n\n\n".to_string()), ol: vec![], ul: vec![], headings: vec![] }] }] }));
  let paths = files.into_iter().map(|(file, literal)| (Path::new(file), literal));
  for (path, lit) in paths {
    let mut s = String::new();
    let mut fd = match File::open(&path) {
      Err(why) => panic!("couldn't open {}: {}", path.display(),
                                                 why.description()),
      Ok(file) => file,
    };
    let mut contents = String::new();
    match fd.read_to_string(&mut contents) {
        Err(why) => panic!("couldn't read {}: {}", path.display(),
                                                   why.description()),
        _        => ()
    }

    let doc = match document(&contents) {
      Ok((_, it)) => it,
      bad         => panic!(format!("failed parsing {}: {:?}", path.display(), bad))
    };
    assert_eq!(doc, lit);
  }
}
*/
