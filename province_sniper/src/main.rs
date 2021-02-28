#[macro_use] extern crate util_macros;
extern crate parse;

use parse::Def;

use std::collections::BTreeSet;
use std::path::Path;
use std::io;
use std::fs;

fn main() {
  if let Err(err) = run() {
    println!("error: {:?}", err);
  };
}

fn run() -> Result<(), Error> {
  let collapse = arg("--collapse");
  let (inverse, (provinces, location)) = match collapse {
    true => (false, (Provinces::Collapse, "collapse")),
    false => (arg("--inverse"), read_provinces()?)
  };

  let definitions = read_definition()?;
  let mut removed: usize = 0;
  let mut new_defs = Vec::new();
  for mut def in definitions {
    if provinces.contains(&def) == inverse {
      def.id = new_defs.len() as u32;
      new_defs.push(def);
    } else {
      removed += 1;
    };
  };

  let new_def = new_defs
    .into_iter()
    .map(|e| e.to_string())
    .collect::<String>();
  println!("writing provinces to definition_new.csv");
  fs::write("definition_new.csv", new_def)?;

  match (collapse, inverse) {
    (true, _) => println!("collapsed provinces"),
    (false, false) => println!("removed all provinces defined in {} ({} provinces)", location, removed),
    (false, true) => println!("removed all provinces except those defined in {} ({} provinces)", location, removed)
  };

  Ok(())
}

#[inline]
fn arg(find: &str) -> bool {
  std::env::args().any(|a| a == find)
}

fn read_definition() -> Result<Vec<Def>, Error> {
  match read("definition.csv") {
    Ok(Some(data)) => match parse::parse_csv(data) {
      Some(data) => Ok(data),
      None => Err("unable to parse definition.csv".into())
    },
    Ok(None) => Err("could not find definition.csv".into()),
    Err(err) => Err(err.into())
  }
}

fn read_provinces() -> Result<(Provinces, &'static str), Error> {
  if let Some(data) = read_any(&["error.log", "error.txt"])? {
    Provinces::parse_from_log(data)
      .map(|provinces| (provinces, "error.log or error.txt"))
      .ok_or("unable to parse error.txt".into())
  } else if let Some(data) = read("provinces.txt")? {
    Provinces::parse_from_list(data)
      .map(|provinces| (provinces, "provinces.txt"))
      .ok_or("unable to parse provinces.txt".into())
  } else if let Some(data) = read("colors.txt")? {
    Provinces::parse_from_colors(data)
      .map(|provinces| (provinces, "colors.txt"))
      .ok_or("unable to parse colors.txt".into())
  } else {
    Err("could not find error.log, error.txt, provinces.txt, or colors.txt".into())
  }
}

fn read<P: AsRef<Path>>(path: P) -> Result<Option<String>, io::Error> {
  match fs::read_to_string(path) {
    Ok(out) => Ok(Some(out)),
    Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
    Err(err) => Err(err)
  }
}

fn read_any<P: AsRef<Path>, S: AsRef<[P]>>(paths: S) -> Result<Option<String>, io::Error> {
  for path in paths.as_ref() {
    if let Some(data) = read(path)? {
      return Ok(Some(data));
    };
  };
  Ok(None)
}

enum Provinces {
  Ids(BTreeSet<u32>),
  Colors(BTreeSet<[u8; 3]>),
  Collapse
}

impl Provinces {
  #[inline]
  fn parse_from_log(data: String) -> Option<Provinces> {
    parse::parse_log(data).map(From::from)
  }

  #[inline]
  fn parse_from_list(data: String) -> Option<Provinces> {
    parse::parse_list(data).map(From::from)
  }

  #[inline]
  fn parse_from_colors(data: String) -> Option<Provinces> {
    parse::parse_colors(data).map(From::from)
  }
}

impl Provinces {
  #[inline]
  fn contains(&self, def: &Def) -> bool {
    match self {
      Provinces::Ids(tree) => tree.contains(&def.id),
      Provinces::Colors(tree) => tree.contains(&def.rgb),
      Provinces::Collapse => true
    }
  }
}

impl From<Vec<u32>> for Provinces {
  #[inline]
  fn from(value: Vec<u32>) -> Provinces {
    Provinces::Ids(value.into_iter().collect())
  }
}

impl From<Vec<[u8; 3]>> for Provinces {
  #[inline]
  fn from(value: Vec<[u8; 3]>) -> Provinces {
    Provinces::Colors(value.into_iter().collect())
  }
}

error_enum!{
  enum Error {
    Io(io::Error),
    Custom(&'static str)
  }
}
