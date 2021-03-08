#[macro_use] extern crate util_macros;
extern crate parse;

use image::{DynamicImage, ImageFormat, RgbImage, Rgb};
use image::codecs::bmp::BmpDecoder;

use rand::distributions::{Standard, Distribution};
use rand::Rng;

use parse::{Def, Kind};

use std::collections::{HashMap, HashSet};
use std::thread::spawn;
use std::path::Path;
use std::sync::Arc;
use std::{fs, io};

fn main() {
  if let Err(err) = run() {
    println!("error: {:?}", err);
  };
}

fn run() -> Result<(), Error> {
  let defs1 = spawn(|| read_defs("definition_1.csv"));
  let defs2 = spawn(|| read_defs("definition_2.csv"));
  println!("reading defs...");

  let defs1 = defs1.join().unwrap()?;
  let defs2 = defs2.join().unwrap()?;
  println!("defs read");

  let provs1 = spawn(|| read_bmp("provinces_1.bmp"));
  let provs2 = spawn(|| read_bmp("provinces_2.bmp"));
  println!("reading images...");
  
  let provs1 = provs1.join().unwrap()?;
  let provs2 = provs2.join().unwrap()?;
  println!("images read");

  println!("{:?} | {:?}", provs1.dimensions(), provs2.dimensions());
  assert_eq!(provs1.width(), provs2.width());

  let mut rng = rand::thread_rng();
  let all_colors = get_common(&provs1, &provs2);
  let replacement = get_replacement_map(&defs1, &defs2, &all_colors, &mut rng);
  println!("replacement colors calculated");
  let common = Arc::new(CommonData { all_colors, replacement });

  let defs_common = common.clone();
  let defs_handle = spawn(move || make_new_defs(defs1, defs2, defs_common));

  let provs_common = common.clone();
  let provs_handle = spawn(move || make_new_provs(provs1, provs2, provs_common));

  defs_handle.join().unwrap()?;
  println!("new defs finished");

  provs_handle.join().unwrap()?;
  println!("new provs finished");

  Ok(())
}

type Replacement = HashMap<([u8; 3], Which), [u8; 3]>;

fn make_new_defs(defs1: Vec<Def>, defs2: Vec<Def>, common: Arc<CommonData>) -> Result<(), Error> {
  let size = defs1.len() + defs2.len();
  let mut new_defs: Vec<Def> = Vec::with_capacity(size);
  new_defs.push(Def::initial());

  for (mut def, which) in iter_defs_marked(defs1, defs2) {
    if def.kind == Kind::Unknown { continue };
    if !common.all_colors.contains(&def.rgb) { continue };

    replace_rgb(&mut def.rgb, which, &common.replacement);
    new_defs.push(def);
  };

  new_defs.sort();

  for (i, def) in new_defs.iter_mut().enumerate() {
    def.id = i;
  };

  let new_defs = new_defs.into_iter()
    .map(|def| def.to_string())
    .collect::<String>();
  fs::write("definition_new.csv", new_defs)?;

  Ok(())
}

fn make_new_provs(provs1: RgbImage, provs2: RgbImage, common: Arc<CommonData>) -> Result<(), Error> {
  let (width, height) = provs1.dimensions();
  let mut new_provs = RgbImage::new(width, height);
  let iter = Iterator::zip(provs1.pixels(), provs2.pixels());
  let iter = Iterator::zip(new_provs.pixels_mut(), iter);
  for (Rgb(new_pixel), (&Rgb(pixel1), &Rgb(pixel2))) in iter {
    *new_pixel = make_new_pixel(pixel1, pixel2, &common);
  };

  new_provs.save_with_format("provinces_new.bmp", ImageFormat::Bmp)?;
  
  Ok(())
}

fn make_new_pixel(pixel1: [u8; 3], pixel2: [u8; 3], common: &CommonData) -> [u8; 3] {
  fn f(rgb: [u8; 3], which: Which, replacement: &Replacement) -> Option<[u8; 3]> {
    if rgb == [0, 0, 0] { return None };
    let rgb = replacement.get(&(rgb, which))
      .cloned().unwrap_or(rgb);
    Some(rgb)
  }

  None
    .or_else(|| f(pixel1, Which::Map1, &common.replacement))
    .or_else(|| f(pixel2, Which::Map2, &common.replacement))
    .unwrap_or([0, 0, 0])
}

struct CommonData {
  all_colors: HashSet<[u8; 3]>,
  replacement: Replacement
}

fn replace_rgb(rgb: &mut [u8; 3], which: Which, replacement: &Replacement) {
  if let Some(&new) = replacement.get(&(*rgb, which)) {
    *rgb = new;
  };
}

fn read_bmp<P: AsRef<Path>>(path: P) -> Result<RgbImage, Error> {
  let img_file = fs::File::open(path)?;
  let img = BmpDecoder::new(img_file)?;
  let img = DynamicImage::from_decoder(img)?;
  Ok(img.into_rgb8())
}

fn read_defs<P: AsRef<Path>>(path: P) -> Result<Vec<Def>, Error> {
  let data = fs::read_to_string(path)?;
  let data =  parse::parse_csv(data)
    .ok_or("failed to parse definitions")?;
  Ok(data)
}

fn get_conflicting<'d>(
  defs1: &'d [Def],
  defs2: &'d [Def]
) -> HashSet<&'d [u8; 3]> {
  let mut colors = HashSet::new();
  for def1 in defs1 {
    for def2 in defs2 {
      if def1.rgb == def2.rgb {
        colors.insert(&def1.rgb);
      };
    };
  };
  colors.shrink_to_fit();
  colors
}

fn get_replacement<'d>(
  defs1: &'d [Def],
  defs2: &'d [Def],
  all_colors: &'d HashSet<[u8; 3]>,
  target: usize,
  rng: &mut impl Rng
) -> HashSet<[u8; 3]> {
  let mut colors = iter_defs(defs1, defs2)
    .filter(|&def| !all_colors.contains(&def.rgb))
    .map(|def| def.rgb.clone())
    .collect::<HashSet<_>>();
  colors.remove(&[0, 0, 0]);
  while colors.len() < target {
    let color = rng.gen::<[u8; 3]>();
    if !all_colors.contains(&color) {
      colors.insert(color);
    };
  };
  colors.shrink_to_fit();
  colors
}

fn get_replacement_map<'d>(
  defs1: &'d [Def],
  defs2: &'d [Def],
  all_colors: &'d HashSet<[u8; 3]>,
  rng: &mut impl Rng
) -> Replacement {
  let conflicting = get_conflicting(defs1, defs2);
  let replacement = get_replacement(defs1, defs2, all_colors, conflicting.len(), rng);
  assert!(replacement.len() >= conflicting.len());
  let conflicting = conflicting.into_iter()
    .map(|color| (color.clone(), rng.gen()));
  let replacement = replacement.into_iter();
  Iterator::zip(conflicting, replacement).collect()
}

fn get_common(prov1: &RgbImage, prov2: &RgbImage) -> HashSet<[u8; 3]> {
  let size = std::cmp::max(prov1.len(), prov2.len());
  let mut colors = HashSet::with_capacity(size);
  colors.extend(prov1.pixels().map(|&Rgb(p)| p));
  colors.extend(prov2.pixels().map(|&Rgb(p)| p));
  colors.remove(&[0, 0, 0]);
  colors.shrink_to_fit();
  colors
}

fn iter_defs<'d>(defs1: &'d [Def], defs2: &'d [Def]) -> impl Iterator<Item = &'d Def> {
  Iterator::chain(defs1.iter(), defs2.iter())
}

fn iter_defs_marked(defs1: Vec<Def>, defs2: Vec<Def>) -> impl Iterator<Item = (Def, Which)> {
  Iterator::chain(
    defs1.into_iter().map(|def| (def, Which::Map1)),
    defs2.into_iter().map(|def| (def, Which::Map2))
  )
}

/*struct Colors<'def> {
  pub conflicting: HashSet<&'def [u8; 3]>,
  pub replacement: HashSet<[u8; 3]>
}

impl<'def> Colors<'def> {
  fn from_data(data: &'def Data, rng: &mut impl Rng) -> Self {
    let colors1 = get_colors(&data.defs1);
    let colors2 = get_colors(&data.defs2);

    let conflicting = HashSet::intersection(&colors1, &colors2)
      .copied().collect::<HashSet<_>>();

    let mut replacement = data.iter_defs()
      .filter(|&def| !data.common.contains(&def.rgb))
      .map(|def| def.rgb.clone()).collect::<HashSet<_>>();
    replacement.remove(&[0, 0, 0]);
    while replacement.len() < conflicting.len() {
      let color = rng.gen::<[u8; 3]>();
      if !data.common.contains(&color) {
        replacement.insert(color);
      };
    };
    replacement.shrink_to_fit();


    
    Colors {
      conflicting,
      replacement
    }
  }
}

struct Replacement<'def, 'colors> {
  pub replacement1: HashMap<(&'def [u8; 3], WhichSet), &'colors [u8; 3]>
}

impl<'def, 'colors> Replacement<'def, 'colors> {
  fn from_colors(colors: &'colors Colors, rng: &mut impl Rng) -> Self {
    let replacement1 = HashMap::new();
    let replacement2 = HashMap::new();


  }
}*/

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum Which {
  Map1,
  Map2
}

impl Distribution<Which> for Standard {
  fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Which {
    if rng.gen() { Which::Map1 } else { Which::Map2 }
  }
}

error_enum!{
  pub enum Error {
    Io(io::Error),
    Image(image::ImageError),
    Custom(&'static str)
  }
}
