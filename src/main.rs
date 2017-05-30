use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::error::Error;
use std::collections::BTreeMap;

use regex::Regex;


extern crate regex;

// Match COPY components '^COPY +--from=([^\s]+) ("[^"]+"|[^\s"]+) +("[^"]+"|[^\s"]+)$'

fn init_stage<'a>(register: &mut Vec<&'a str>, command: &'a str) -> &'a str {
    let capture = Regex::new(r"^FROM +[^\s]+ +as +([^\s]+)$").unwrap()
        .captures(command);

    match capture {
        Some(x) => register.push(x.get(1).unwrap().as_str()),
        None => (),
    };

    return command
}


fn main() {
    let dockerfile = String::from_utf8(load_file("./test/Dockerfile").into_vec())
        .unwrap_or(String::from(""));

    let mut split: Vec<&str> = dockerfile
        .split_terminator("\n")
        .collect();

    split.retain(|x| !x.is_empty());

    let command_reg = Regex::new(r"^([A-Z]+)").unwrap();

    let mut stages: Vec<&str> = vec![];
    let mut stages_register: BTreeMap<&str, Vec<&str>> = BTreeMap::new();

    // Just incase there is white space at the start and end of the line
    let from_and_copy: Vec<&str> = split.into_iter()
        .map(|x| x.trim())
        .filter(|x| !x.starts_with("#")) // remove comments
        .map(|x| {
            let stage = match command_reg.captures(x).unwrap().get(1).unwrap().as_str() {
                "FROM" => {
                    init_stage(&mut stages, x);
                    stages_register.insert(stages.last().unwrap(), vec![]);
                    x
                },
                "COPY" => {
                    // TODO: Implement this stuff...
                }
                _ => x
            };
            stages_register.get_mut(stages.last().unwrap()).unwrap().push(stage);
            x
        })
        .collect();

    println!("{:?}", stages_register);
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
