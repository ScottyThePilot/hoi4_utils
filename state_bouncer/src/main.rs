#[macro_use] extern crate lazy_static;
#[macro_use] extern crate util_macros;
extern crate notify;
extern crate paths;
extern crate regex;
extern crate ctrlc;

use notify::{Watcher, RecursiveMode, DebouncedEvent, watcher};
use regex::Regex;

use crate::paths::Paths;

use std::sync::mpsc::{Receiver, channel};
use std::path::{PathBuf, Path};
use std::time::Duration;
use std::ffi::OsStr;
use std::fs;

fn main() {
  if let Err(err) = run() {
    println!("error: {:?}", err);
  };
}

fn run() -> Result<(), Error> {
  let paths = Paths::resolve()?;
  println!("{}", paths);
  clean_dir(&paths.state_src)?;
  println!("watching for changes...");
  watch_for_changes(&paths)?;
  Ok(())
}

fn clean_dir(path: impl AsRef<Path>) -> Result<(), Error> {
  for file in fs::read_dir(&path)? {
    let file = file?.path();
    fs::remove_file(&file)?;
    let file = file.strip_prefix(&path).unwrap();
    println!("erased file: {}", file.display());
  };

  Ok(())
}

fn watch_for_changes(paths: &Paths) -> Result<(), Error> {
  let (kill_tx, kill_rx) = channel();
  ctrlc::set_handler(move || kill_tx.send(()).unwrap())?;
  
  let (tx, rx) = channel();
  let mut watcher = watcher(tx, Duration::from_millis(100))?;
  watcher.watch(&paths.state_src, RecursiveMode::NonRecursive)?;
  watcher.watch(&paths.loc_src, RecursiveMode::NonRecursive)?;

  loop {
    if try_recv(&kill_rx)?.is_some() {
      println!("exiting...");
      watcher.unwatch(&paths.state_src)?;
      watcher.unwatch(&paths.loc_src)?;
      return Ok(());
    };

    let mut loc_changes = 0;
    for path in rx.try_iter() {
      let path = transform_event(path)?;
      let path = try_continue!(path);
      if path.starts_with(&paths.state_src) {
        let file = path.file_name().unwrap();
        let dest_file = transform_name(file);
        let file = Path::new(file);
        let mut dest = paths.state_dest.clone();
        dest.push(dest_file);
        fs::rename(&path, dest)?;
        println!("moved file: {}", file.display());
      } else if path == paths.loc_src {
        loc_changes += 1;
      };
    };

    if loc_changes > 0 {
      println!("localisation change(s): {}", loc_changes);
    };
  };
}

fn try_recv<T>(rx: &Receiver<T>) -> Result<Option<T>, Error> {
  use std::sync::mpsc::TryRecvError::*;
  match rx.try_recv() {
    Ok(t) => Ok(Some(t)),
    Err(Empty) => Ok(None),
    Err(Disconnected) => Err("sender disconnected".into())
  }
}

fn transform_event(event: DebouncedEvent) -> Result<Option<PathBuf>, Error> {
  match event {
    DebouncedEvent::Create(src) |
    DebouncedEvent::Write(src) => Ok(Some(src)),
    DebouncedEvent::Error(err, _) => Err(err.into()),
    _ => Ok(None)
  }
}

fn transform_name(name: &OsStr) -> String {
  lazy_static!{
    static ref RX: Regex = Regex::new(r"(\d+)-STATE_\d+\.txt").unwrap();
  }
  let name = name.to_str().unwrap();
  RX.replace(name, "$1-State.txt").to_string()
}

error_enum!{
  pub enum Error {
    Io(std::io::Error),
    CtrlC(ctrlc::Error),
    Watcher(notify::Error),
    Custom(&'static str)
  }
}
