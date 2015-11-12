#[macro_use]
extern crate mime;

extern crate iron;
extern crate persistent;
extern crate router;
extern crate urlencoded;
extern crate staticfile;
extern crate mount;

extern crate image;
extern crate itertools;
extern crate rustc_serialize;

extern crate gmagick;

use std::fs::File;
use std::path::{Path};
use std::io::Cursor;
use image::{GenericImage};

mod web;
mod utils;

fn main() {
  let task: &str = &std::env::args()
    .collect::<Vec<_>>()
    .get(1)
    .unwrap()
    .clone();
    
  match task {
    "web:run" => { web::run(); },
    _ => { println!("error"); }
  }
}
