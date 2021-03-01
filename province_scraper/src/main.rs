#[macro_use] extern crate util_macros;
extern crate image;

use image::{DynamicImage, RgbImage, Rgb};
use image::codecs::{bmp::BmpDecoder, png::PngDecoder};

use std::collections::HashSet;
use std::{fs, io};

fn main() {
  run().unwrap();
}

fn run() -> Result<(), Error> {
  let img = any(&[read_bmp, read_png][..])?;
  println!("image loaded");

  let mut colors: HashSet<&Rgb<u8>> = img.pixels().collect();
  colors.remove(&Rgb([0, 0, 0]));
  colors.remove(&Rgb([255, 255, 255]));
  println!("colors extracted");

  let color_count = colors.len();

  let mut colors = colors.into_iter()
    .map(|&Rgb([r, g, b])| format!("[{},{},{}] ", r, g, b))
    .collect::<String>();
  colors.pop(); // Remove last whitespace

  fs::write("colors.txt", colors)?;
  println!("colors written to colors.txt ({} colors)", color_count);

  Ok(())
}

fn read_bmp() -> Result<RgbImage, Error> {
  let img_file = fs::File::open("provinces.bmp")?;
  let img = BmpDecoder::new(img_file)?;
  let img = DynamicImage::from_decoder(img)?;
  Ok(img.into_rgb8())
}

fn read_png() -> Result<RgbImage, Error> {
  let img_file = fs::File::open("provinces.png")?;
  let img = PngDecoder::new(img_file)?;
  let img = DynamicImage::from_decoder(img)?;
  Ok(img.into_rgb8())
}

fn any<T, E, F>(funcs: &[F]) -> Result<T, Vec<E>>
where F: Fn() -> Result<T, E> {
  let mut errors = Vec::new();

  for f in funcs {
    match f() {
      Ok(t) => return Ok(t),
      Err(t) => errors.push(t.into())
    };
  };

  Err(errors)
}

error_enum!{
  pub enum Error {
    Io(io::Error),
    Image(image::ImageError),
    Custom(&'static str),
    Many(Vec<Error>)
  }
}
