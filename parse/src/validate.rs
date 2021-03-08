use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::error::Error;
use std::{fs, fmt, io};

use crate::definition::*;

//pub type ValidateError = (Option<(String, String)>, Vec<String>);

#[derive(Debug)]
pub struct ValidateError {
  pub write_result: Result<(), io::Error>,
  pub invalid_items: Vec<String>
}

impl fmt::Display for ValidateError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if let Err(write_error) = &self.write_result {
      writeln!(f, "{}", write_error);
    };

    for invalid in &self.invalid_items {
      write!(f, "\n{}", invalid);
    };

    Ok(())
  }
}

impl Error for ValidateError {}

pub fn validate_defs(defs: &[Def], dump: bool) -> Result<(), ValidateError> {
  let mut ids: HashMap<usize, &Def> = HashMap::new();
  let mut colors: HashMap<[u8; 3], &Def> = HashMap::new();
  let mut errors: Vec<String> = Vec::new();
  let mut duplicate_colors = HashSet::new();

  for def in defs {
    ent(&mut ids, &def.id, &def, |ent| {
      err_duplicate_thing(&mut errors, "id", ent, def);
    });

    ent(&mut colors, &def.rgb, &def, |ent| {
      duplicate_colors.insert(ent.rgb);
      err_duplicate_thing(&mut errors, "color", ent, def);
    });
  };

  if errors.is_empty() {
    Ok(())
  } else {
    let write_result = match dump {
      true => dump_duplicate_colors(duplicate_colors, colors),
      false => Ok(())
    };
    
    Err(ValidateError {
      write_result,
      invalid_items: errors
    })
  }
}

fn dump_duplicate_colors(
  duplicate_colors: HashSet<[u8; 3]>,
  colors: HashMap<[u8; 3], &Def>
) -> Result<(), io::Error> {
  let map_fn = |[r, g, b]: &[u8; 3]| format!("[{},{},{}] ", r, g, b);
  let mut conflicting_colors = duplicate_colors.iter()
    .map(map_fn).collect::<String>();
  conflicting_colors.pop();
  let mut regular_colors = colors.keys()
    .filter(|color| !duplicate_colors.contains(*color))
    .map(map_fn).collect::<String>();
  regular_colors.pop();

  fs::write("dump_colors_conflicting.txt", conflicting_colors)?;
  fs::write("dump_colors.txt", regular_colors)?;

  Ok(())
}

fn ent<'d, T, F>(map: &mut HashMap<T, &'d Def>, t: &T, def: &'d Def, mut f: F)
where T: Eq + Hash + Clone, F: FnMut(&'d Def) {
  use std::collections::hash_map::Entry;
  match map.entry(t.clone()) {
    Entry::Occupied(entry) => {
      let entry = *entry.get();
      f(entry);
    },
    Entry::Vacant(entry) => {
      entry.insert(def);
    }
  };
}

#[inline]
fn err_duplicate_thing(errors: &mut Vec<String>, thing: &str, def1: &Def, def2: &Def) {
  errors.push(format!(
    "duplicate {}s exist: {}={:?}, {}={:?}",
    thing,
    def1.id, def1.rgb,
    def2.id, def2.rgb
  ));
}
