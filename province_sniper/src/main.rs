#[macro_use] extern crate util_macros;
extern crate parse;

use parse::{Def, Kind};

use std::collections::BTreeSet;
use std::path::Path;
use std::{io, fs, fmt};

fn main() {
  match run() {
    Err(Error::Validation(errors)) => println!("error: validation failed\n{}", errors),
    Err(err) => println!("error: {:?}", err),
    _ => {}
  };
}

fn run() -> Result<(), Error> {
  let rule = Rule::open()?;
  println!("definition rule: {}", rule);

  let defs = read_definition()?;
  println!("definitions read from definition.csv ({} provinces)", defs.len());

  conditional_validation(&defs, arg("--validate"))?;

  let (defs, removed) = create_definitions(defs, |def| rule.apply(def));
  println!("new definitions created, {} provinces removed", removed);

  conditional_validation(&defs, arg("--post-validate"))?;

  write_definition(&defs)?;
  println!("new definitions written to definition_new.csv ({} provinces)", defs.len());

  Ok(())
}

#[inline]
fn arg(find: &str) -> bool {
  std::env::args().skip(1).any(|a| a == find)
}

fn conditional_validation(definitions: &[Def], condition: bool) -> Result<(), Error> {
  if condition {
    if let Err((dump, errors)) = parse::validate_defs(&definitions, arg("--dump-validate")) {
      if let Some((dump_colors_conflicting, dump_colors)) = dump {
        fs::write("dump_colors_conflicting.txt", dump_colors_conflicting)?;
        fs::write("dump_colors.txt", dump_colors)?;
        println!("check dump_colors_conflicting.txt and dump_colors.txt for color data");
      };

      return Err(errors.into());
    };

    println!("no duplicate ids or colors");
  } else {
    println!("no validation performed");
  };

  Ok(())
}

fn create_definitions<F>(definitions: Vec<Def>, mut func: F) -> (Vec<Def>, usize)
where F: FnMut(&mut Def) -> bool {
  let mut removed = 0;
  let mut new_definitions = Vec::new();
  let keep_lakes = arg("--keep-lakes");
  new_definitions.push(Def::initial());
  for mut def in definitions {
    if def.kind != Kind::Unknown {
      if func(&mut def) || (def.kind != Kind::Lake && keep_lakes) {
        //def.id = new_definitions.len();
        new_definitions.push(def);
      } else {
        removed += 1;
      };
    };
  };

  new_definitions.sort();

  for (i, def) in new_definitions.iter_mut().enumerate() {
    def.id = i;
  };
  
  (new_definitions, removed)
}

fn write_definition(defs: &[Def]) -> Result<(), Error> {
  let defs = defs.iter().map(|e| e.to_string()).collect::<String>();
  fs::write("definition_new.csv", defs).map_err(From::from)
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

#[derive(Debug)]
enum Rule {
  Whitelist(Criteria),
  Blacklist(Criteria),
  Always
}

impl Rule {
  fn apply(&self, def: &Def) -> bool {
    match self {
      Rule::Whitelist(criteria) => criteria.contains(def),
      Rule::Blacklist(criteria) => !criteria.contains(def),
      Rule::Always => true
    }
  }

  fn open() -> Result<Rule, Error> {
    if arg("--collapse") {
      Ok(Rule::Always)
    } else {
      let criteria = Criteria::open()?;
      if arg("--whitelist") {
        // Remove all not included in the criteria
        Ok(Rule::Whitelist(criteria))
      } else {
        // Remove all included in the criteria
        Ok(Rule::Blacklist(criteria))
      }
    }
  }
}

impl fmt::Display for Rule {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Rule::Whitelist(Criteria::Ids(_, loc)) => 
        write!(f, "Rule(ONLY province ids IN {})", loc),
      Rule::Whitelist(Criteria::Colors(_, loc)) => 
        write!(f, "Rule(ONLY province colors IN {})", loc),
      Rule::Blacklist(Criteria::Ids(_, loc)) =>
        write!(f, "Rule(ONLY province ids NOT IN {})", loc),
      Rule::Blacklist(Criteria::Colors(_, loc)) =>
        write!(f, "Rule(ONLY province colors NOT IN {})", loc),
      Rule::Always =>
        write!(f, "Rule(ANY/COLLAPSE)")
    }
  }
}

#[derive(Debug)]
enum Criteria {
  Ids(BTreeSet<usize>, &'static str),
  Colors(BTreeSet<[u8; 3]>, &'static str)
}

impl Criteria {
  fn open() -> Result<Criteria, Error> {
    if let Some(data) = read_any(&["error.log", "error.txt"])? {
      Criteria::parse_from_log(data, "error.log or error.txt")
        .ok_or("unable to parse error.txt".into())
    } else if let Some(data) = read("provinces.txt")? {
      Criteria::parse_from_list(data, "provinces.txt")
        .ok_or("unable to parse provinces.txt".into())
    } else if let Some(data) = read("colors.txt")? {
      Criteria::parse_from_colors(data, "colors.txt")
        .ok_or("unable to parse colors.txt".into())
    } else {
      Err("could not find error.log, error.txt, provinces.txt, or colors.txt".into())
    }
  }

  #[inline]
  fn parse_from_log(data: String, location: &'static str) -> Option<Criteria> {
    parse::parse_log(data).map(|data| {
      let data = data.into_iter()
        .collect::<BTreeSet<_>>();
      Criteria::Ids(data, location)
    })
  }

  #[inline]
  fn parse_from_list(data: String, location: &'static str) -> Option<Criteria> {
    parse::parse_list(data).map(|data| {
      let data = data.into_iter()
        .collect::<BTreeSet<_>>();
      Criteria::Ids(data, location)
    })
  }

  #[inline]
  fn parse_from_colors(data: String, location: &'static str) -> Option<Criteria> {
    parse::parse_colors(data).map(|data| {
      let data = data.into_iter()
        .collect::<BTreeSet<_>>();
      Criteria::Colors(data, location)
    })
  }
}

impl Criteria {
  #[inline]
  fn contains(&self, def: &Def) -> bool {
    match self {
      Criteria::Ids(tree, _) => tree.contains(&def.id),
      Criteria::Colors(tree, _) => tree.contains(&def.rgb)
    }
  }
}

error_enum!{
  enum Error {
    Io(io::Error),
    Validation(Validation),
    Custom(&'static str)
  }
}

impl From<Vec<String>> for Error {
  fn from(inner: Vec<String>) -> Error {
    Error::Validation(Validation { inner })
  }
}

#[derive(Debug)]
struct Validation {
  inner: Vec<String>
}

impl fmt::Display for Validation {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.inner.join("\n"))
  }
}
