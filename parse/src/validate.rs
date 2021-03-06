use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::definition::*;

pub type ValidateError = (Option<(String, String)>, Vec<String>);

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
    let dump = match dump {
      true => Some(dump_duplicate_colors(duplicate_colors, colors)),
      false => None
    };
    
    Err((dump, errors))
  }
}

fn dump_duplicate_colors(duplicate_colors: HashSet<[u8; 3]>, colors: HashMap<[u8; 3], &Def>) -> (String, String) {
  let map_fn = |[r, g, b]: &[u8; 3]| format!("[{},{},{}] ", r, g, b);
  let mut offending_colors = duplicate_colors.iter()
    .map(map_fn).collect::<String>();
  offending_colors.pop();
  let mut regular_colors = colors.keys()
    .filter(|color| !duplicate_colors.contains(*color))
    .map(map_fn).collect::<String>();
  regular_colors.pop();
  (offending_colors, regular_colors)
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
