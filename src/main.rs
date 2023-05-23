use glimmer_swc::{gjs_to_js, Options};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Must pass input filename");
        exit(-1);
    }
    let filename: PathBuf = args[1].clone().into();

    let src = fs::read_to_string(filename.clone()).unwrap();

    let output = gjs_to_js(src, Options { filename: Some(filename) }).expect("converted");
    println!("{}" , output);
}
