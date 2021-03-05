#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::{Regex, Captures};

use std::cmp::{Ord, PartialOrd, Ordering};
use std::str::FromStr;
use std::fmt;

lazy_static!{
  static ref RX_CSV: Regex = Regex::new(r"(\d+);(\d+);(\d+);(\d+);(\w+);(true|false);(\w+);(\d+)").unwrap();
  static ref RX_LOG: Regex = Regex::new(r"\[.+\]\[.+\]: Province (\d+) has no pixels in provinces\.bmp").unwrap();
  static ref RX_COLOR: Regex = Regex::new(r"\[(\d+),(\d+),(\d+)\]").unwrap();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
  Unknown = 0,
  Land = 1,
  Ocean = 2,
  Lake = 3
}

impl Kind {
  #[inline]
  pub fn as_str(&self) -> &'static str {
    match self {
      Kind::Unknown => "unknown",
      Kind::Land => "land",
      Kind::Ocean => "ocean",
      Kind::Lake => "lake"
    }
  }

  #[inline]
  pub fn is(&self, other: &str) -> bool {
    self.as_str() == other
  }
}

impl FromStr for Kind {
  type Err = ();

  fn from_str(s: &str) -> Result<Kind, ()> {
    match s {
      "unknown" => Ok(Kind::Unknown),
      "land" => Ok(Kind::Land),
      "ocean" => Ok(Kind::Ocean),
      "lake" => Ok(Kind::Lake),
      _ => Err(())
    }
  }
}

impl fmt::Display for Kind {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{}", self.as_str())
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Def {
  pub id: usize,
  pub rgb: [u8; 3],
  pub kind: Kind,
  pub coastal: bool,
  pub terrain: String,
  pub continent: u32
}

impl Def {
  pub fn initial() -> Def {
    Def {
      id: 0,
      rgb: [0, 0, 0],
      kind: Kind::Land,
      coastal: false,
      terrain: "unknown".to_owned(),
      continent: 0
    }
  }

  pub fn is_initial(&self) -> bool {
    self.id == 0 &&
    self.rgb == [0, 0, 0] &&
    self.kind == Kind::Land &&
    self.coastal == false &&
    self.terrain == "unknown" &&
    self.continent == 0
  }

  
}

impl PartialOrd for Def {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(Kind::cmp(&self.kind, &other.kind))
  }
}

impl Ord for Def {
  fn cmp(&self, other: &Self) -> Ordering {
    Kind::cmp(&self.kind, &other.kind)
  }
}

impl fmt::Display for Def {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{};{};{};{};{};{};{};{}\n",
      self.id,
      self.rgb[0],
      self.rgb[1],
      self.rgb[2],
      self.kind,
      self.coastal,
      self.terrain,
      self.continent
    )
  }
}

impl FromStr for Def {
  type Err = ();

  fn from_str(s: &str) -> Result<Def, ()> {
    parse_csv_line(s).ok_or(())
  }
}

pub fn parse_csv(content: impl AsRef<str>) -> Option<Vec<Def>> {
  let content = content.as_ref().trim();
  content.split_whitespace()
    .map(parse_csv_line)
    .collect()
}

pub fn parse_csv_simple<'a>(content: impl AsRef<str>) -> Option<Vec<(usize, String)>> {
  let content = content.as_ref().trim();
  content.split_whitespace()
    .map(parse_csv_line_simple)
    .collect()
}

#[inline]
fn parse_csv_line(line: &str) -> Option<Def> {
  let captures = RX_CSV.captures(line)?;
  Some(Def {
    id: par(&captures, 1)?,
    rgb: [
      par(&captures, 2)?,
      par(&captures, 3)?,
      par(&captures, 4)?
    ],
    kind: par(&captures, 5)?,
    coastal: par(&captures, 6)?,
    terrain: own(&captures, 7),
    continent: par(&captures, 8)?
  })
}

#[inline]
fn par<F: FromStr>(cap: &Captures, i: usize) -> Option<F> {
  cap.get(i).unwrap().as_str().parse::<F>().ok()
}

#[inline]
fn own(cap: &Captures, i: usize) -> String {
  cap.get(i).unwrap().as_str().to_owned()
}

#[inline]
fn parse_csv_line_simple(line: &str) -> Option<(usize, String)> {
  let mut number = String::new();
  let mut rest = String::new();

  let mut split = false;
  for ch in line.chars() {
    if split {
      rest.push(ch);
    } else {
      if ch.is_ascii_digit() {
        number.push(ch);
      } else {
        split = true;
        rest.push(ch);
      };
    };
  };

  let number = number.parse::<usize>().ok()?;
  Some((number, rest))
}

pub fn parse_log(content: impl AsRef<str>) -> Option<Vec<usize>> {
  let out = RX_LOG.captures_iter(content.as_ref())
    .filter_map(|cap| par::<usize>(&cap, 1))
    .collect::<Vec<usize>>();
  if out.is_empty() { None } else { Some(out) }
}

pub fn parse_list(content: impl AsRef<str>) -> Option<Vec<usize>> {
  let content = content.as_ref().trim();
  content.split_whitespace()
    .map(|e| e.parse::<usize>().ok())
    .collect()
}

pub fn parse_colors(content: impl AsRef<str>) -> Option<Vec<[u8; 3]>> {
  let content = content.as_ref().trim();
  let out = content.split_whitespace()
    .filter_map(parse_color_chunk)
    .collect::<Vec<_>>();
  if out.is_empty() { None } else { Some(out) }
}

fn parse_color_chunk(s: &str) -> Option<[u8; 3]> {
  let captures = RX_COLOR.captures(s)?;
  Some([
    par(&captures, 1)?,
    par(&captures, 2)?,
    par(&captures, 3)?
  ])
}

pub fn validate_defs(defs: &[Def]) -> Result<(), Vec<String>> {
  use std::collections::hash_map::{HashMap, Entry};
  let mut ids: HashMap<usize, &Def> = HashMap::new();
  let mut colors: HashMap<[u8; 3], &Def> = HashMap::new();
  let mut errors: Vec<String> = Vec::new();

  for def in defs {
    match ids.entry(def.id) {
      Entry::Occupied(entry) => {
        let err = format!(
          "duplicate ids exist: {}, {}",
          entry.get().id,
          def.id
        );
        errors.push(err);
      },
      Entry::Vacant(entry) => {
        entry.insert(def);
      }
    };

    match colors.entry(def.rgb) {
      Entry::Occupied(entry) => {
        let err = format!(
          "duplicate colors exist: {}={:?}, {}={:?}",
          entry.get().id, entry.get().rgb,
          def.id, def.rgb
        );
        errors.push(err);
      },
      Entry::Vacant(entry) => {
        entry.insert(def);
      }
    };
  };

  if errors.is_empty() {
    Ok(())
  } else {
    Err(errors)
  }
}
