use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::error::Error;

use regex::Regex;


extern crate regex;

// Match COPY components '^ *COPY +--from=([^\s]*) ("[^"]*"|[^\s"]*) *("[^"]*"|[^\s"]*)$'

fn main() {
    let dockerfile = String::from_utf8(load_file("./test/Dockerfile").into_vec())
        .unwrap_or(String::from(""));

    let mut split: Vec<&str> = dockerfile
        .split_terminator("\n")
        .collect();

    split.retain(|x| !x.is_empty());

    let command_reg = Regex::new(r"^([A-Z]+)").unwrap();

    // Just incase there is white space at the start and end of the line
    let from_and_copy: Vec<&str> = split.into_iter()
        .map(|x| x.trim())
        .filter(|x| !x.starts_with("#")) // remove comments
        .map(|x| {
            match command_reg.captures(x).unwrap().get(1).unwrap().as_str() {
                "FROM" => x,
                "COPY" => x,
                _ => x
            }
        })
        .collect();

    println!("{:?}", from_and_copy);
}



fn load_file(path: &str) -> Box<[u8]> {
    let fpath = Path::new(path);
    let mut buffer = Vec::new();

    match File::open(&fpath) {
        Err(x) => panic!("Error loading the file: {}", x.description()),
        Ok(mut x) => x.read_to_end(&mut buffer).unwrap(),
    };

    buffer.into_boxed_slice()
}
