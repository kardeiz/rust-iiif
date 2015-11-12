use iron::prelude::*;
use iron::{status};
use iron::{Handler, BeforeMiddleware, AfterMiddleware};

use iron::mime::{Mime};
use iron::typemap::Key;

use mount::Mount;
use staticfile::Static;
use router::Router;
use urlencoded::UrlEncodedQuery;
use persistent::{Write,Read};

use std::path::{Path, PathBuf};
use std::env;
use std::io::Cursor;

use std::collections::HashMap;

use image::{self, GenericImage, DynamicImage};

use itertools::Itertools;

use std::thread;
use std::fs;

use std::collections::BTreeMap;
use rustc_serialize::json::{self, Json, ToJson};

use utils;
use gmagick;


#[derive(Copy, Clone)]
pub struct ImageCache;

impl Key for ImageCache { 
  // type Value = HashMap<String, Vec<u8>>;
  type Value = HashMap<String, DynamicImage>;
}

#[derive(Copy, Clone)]
pub struct Sleeper;

impl Key for Sleeper { type Value = usize; }

enum ImageOps {
  Crop(i64, i64, u64, u64),
  Resize(Option<u64>, Option<u64>),
  Scale(u64, u64),
  Rotate(f64),
  Mirror
}

fn info(req: &mut Request) -> IronResult<Response> {

  let router = req.extensions.get::<Router>().unwrap();
  let id = router.find("id").unwrap();
  
  let mut img = gmagick::Image::from_path(
    Path::new("images").join(&id).to_str().unwrap()).unwrap();

  let (w, h) = img.dimensions();

  let mut base = BTreeMap::new();

  base.insert("@context".to_string(), 
    "http://iiif.io/api/image/2/context.json".to_json() );
  base.insert("@id".to_string(),
    format!("/image/{}", &id).to_json() );
  base.insert("protocol".to_string(),
    "http://iiif.io/api/image".to_json() );
  base.insert("width".to_string(),
    w.to_json() );
  base.insert("height".to_string(),
    h.to_json() );
  base.insert("profile".to_string(),
    vec!["http://iiif.io/api/image/2/level2.json".to_json()].to_json());

  let mut tiles = Vec::new();

  {
    let mut base = BTreeMap::new();
    base.insert("scaleFactors".to_string(), vec![ 1, 2, 4, 8, 16, 32 ]
      .iter()
      .map(|&x| x.to_json() )
      .collect::<Vec<_>>()
      .to_json() );
    base.insert("width".to_string(), 1024.to_json() );
    tiles.push(base.to_json());
  } 

  base.insert("tiles".to_string(), tiles.to_json());

  Ok(Response::with((
    "application/json".parse::<Mime>().unwrap(),
    status::Ok, 
    Json::Object(base).to_string() )))
}

fn image(req: &mut Request) -> IronResult<Response> {
  
  let path = Path::new("images");
  
  let file_name = utils::encode_uri(req.url.path.join("/"), false);
  let file_path = path.join(&file_name);

  if let Ok(metadata) = fs::metadata(&file_path) {
    if metadata.is_file() {
      return Ok(Response::with(( status::Ok, &file_path as &Path )));
    }
  }

  let router = req.extensions.get::<Router>().unwrap();

  let mut image_ops = Vec::new();

  let region = router.find("region").unwrap();

  if region != "full" {
    let regions: Vec<_> = region.split(',').collect();
    let x = regions.get(0).unwrap().parse::<i64>().unwrap();
    let y = regions.get(1).unwrap().parse::<i64>().unwrap();
    let w = regions.get(2).unwrap().parse::<u64>().unwrap();
    let h = regions.get(3).unwrap().parse::<u64>().unwrap();
    image_ops.push(ImageOps::Crop(x, y, w, h));
  }

  let size = router.find("size").unwrap();

  if size != "full" {
    let sizes: Vec<_> = size.split(',').collect();
    let w = {
      let w = sizes.get(0).unwrap();
      if w == &"" { None } 
        else { Some(w.parse::<u64>().unwrap()) }
    };
    let h = {
      let h = sizes.get(1).unwrap();
      if h == &"" { None } 
        else { Some(h.parse::<u64>().unwrap()) }
    };
    image_ops.push(ImageOps::Resize(w, h));
  }

  let rotation = router.find("rotation").unwrap();

  if rotation != "0" {
    let r = rotation.parse::<f64>().unwrap();
    image_ops.push(ImageOps::Rotate(r));
  }

  let id = router.find("id").unwrap();

  let base = gmagick::Image::from_path(
    path.join(&id).to_str().unwrap());

  let mut img = image_ops.iter().fold(base, |acc, item|
    acc.and_then(|mut i| 
      match *item {
        ImageOps::Crop(x, y, w, h) => i.crop(x, y, w, h),
        ImageOps::Resize(w, h) => i.resize(w, h),
        ImageOps::Rotate(r) => i.rotate(r),
        _ => Some(i)
      }
    )
  ).unwrap();

  img.write(&file_path.to_str().unwrap()).unwrap();


  Ok(Response::with((status::Ok, file_path) ))
}




pub fn run() {

  let static_path = env::var("STATIC_PATH")
    .ok()
    .unwrap_or("static/".to_string());

  let port = env::var("PORT")
    .ok()
    .and_then(|s| s.parse::<u16>().ok() )
    .unwrap_or(3000);
  
  let mut router = Router::new();

  router.get("/image/:id/:region/:size/:rotation/:quality.:format", image);
  router.get("/info/:id/info.json", info);

  let mut mount = Mount::new();
  mount.mount("/", router);

  mount.mount("/static", Static::new(Path::new(&static_path)));

  let mut middleware = Chain::new(mount); 

  Iron::new(middleware).http( ("0.0.0.0", port) ).unwrap();

}