#[macro_use] extern crate lazy_static;
extern crate regex;

mod validate;
mod definition;

pub use crate::validate::*;
pub use crate::definition::*;

#[macro_export]
macro_rules! parallelize {
  [$(let $name:ident = $job:expr;)*] => {
    $(let $name = std::thread::spawn($job);)*
    $(let $name = $name.join().unwrap();)*
  };
}
