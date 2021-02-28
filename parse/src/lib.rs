#[macro_use] extern crate lazy_static;
extern crate regex;

use regex::{Regex, Captures};

use std::str::FromStr;

lazy_static!{
  static ref RX_CSV: Regex = Regex::new(r"(\d+);(\d+);(\d+);(\d+);(\w+);(true|false);(\w+);(\d+)").unwrap();
  static ref RX_LOG: Regex = Regex::new(r"\[.+\]\[.+\]: Province (\d+) has no pixels in provinces\.bmp").unwrap();
  static ref RX_COLOR: Regex = Regex::new(r"\[(\d+),(\d+),(\d+)\]").unwrap();
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Def {
  pub id: u32,
  pub rgb: [u8; 3],
  pub kind: String,
  pub coastal: bool,
  pub terrain: String,
  pub continent: u32
}

impl ToString for Def {
  fn to_string(&self) -> String {
    format!(
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

pub fn parse_csv_simple<'a>(content: impl AsRef<str>) -> Option<Vec<(u32, String)>> {
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
    kind: own(&captures, 5),
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
fn parse_csv_line_simple(line: &str) -> Option<(u32, String)> {
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

  let number = number.parse::<u32>().ok()?;
  Some((number, rest))
}

pub fn parse_log(content: impl AsRef<str>) -> Option<Vec<u32>> {
  let out = RX_LOG.captures_iter(content.as_ref())
    .filter_map(|cap| par::<u32>(&cap, 1))
    .collect::<Vec<u32>>();
  if out.is_empty() { None } else { Some(out) }
}

pub fn parse_list(content: impl AsRef<str>) -> Option<Vec<u32>> {
  let content = content.as_ref().trim();
  content.split_whitespace()
    .map(|e| e.parse::<u32>().ok())
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
