use std::path::{Path, PathBuf};
use std::fmt::{self, Display};
use std::env::args;
use std::fs;

#[derive(Debug)]
pub struct Paths {
  pub mod_id: String,
  pub hoi4: PathBuf,
  pub state_src: PathBuf,
  pub state_dest: PathBuf,
  pub loc_src: PathBuf,
  pub loc_dest: PathBuf
}

impl Paths {
  pub fn resolve() -> Result<Paths, &'static str> {
    let mod_id = get_mod_id().ok_or("unable to get mod id")?;
    let hoi4 = get_hoi4().ok_or("unable to get user directory")?;
    let state_src = get_state_src(&hoi4);
    let state_dest = get_state_dest(&hoi4, &mod_id);
    let loc_src = get_loc_src(&hoi4);
    let loc_dest = get_loc_dest(&hoi4, &mod_id);
    Ok(Paths {
      mod_id,
      hoi4,
      state_src,
      state_dest,
      loc_src,
      loc_dest
    })
  }
}

impl Display for Paths {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(
      f, "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}",
      "mod_id", self.mod_id,
      "hoi4", self.hoi4.display(),
      "state_src", self.state_src.display(),
      "state_dest", self.state_dest.display(),
      "loc_src", self.loc_src.display(),
      "loc_dest", self.loc_dest.display()
    )
  }
}

fn get_state_src(hoi4: &Path) -> PathBuf {
  let mut path = hoi4.to_owned();
  path.push("history/states");
  path
}

fn get_state_dest(hoi4: &Path, mod_id: &str) -> PathBuf {
  let mut path = hoi4.to_owned();
  path.push("mod");
  path.push(mod_id);
  path.push("history/states");
  path
}

fn get_loc_src(hoi4: &Path) -> PathBuf {
  let mut path = hoi4.to_owned();
  path.push("localisation/state_names_l_english.yml");
  path
}

fn get_loc_dest(hoi4: &Path, mod_id: &str) -> PathBuf {
  let mut path = hoi4.to_owned();
  path.push("mod");
  path.push(mod_id);
  path.push("localisation/state_names_l_english.yml");
  path
}

fn get_hoi4() -> Option<PathBuf> {
  let mut path = dirs::home_dir()?;
  path.push("Documents/Paradox Interactive/Hearts of Iron IV");
  Some(path)
}

fn get_mod_id() -> Option<String> {
  None
    .or_else(|| args().nth(1))
    .or_else(|| fs::read_to_string("mod_id").ok())
    .map(|mod_id| mod_id.trim().to_owned())
}
