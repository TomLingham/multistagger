use std::collections::BTreeMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{ self, Read, Write, BufRead, BufReader };
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::io::prelude::*;

use rocker::Rocker;
use rocker::DockerCommand;

use regex::Regex;

extern crate regex;
extern crate rocker;

#[macro_use]
extern crate lazy_static;


fn initialize_stage<'a>(register: &mut Vec<&'a str>, command: &'a str) -> () {
    let capture = regex_docker_from.captures(command);

    match capture {
        Some(matches) => register.push(matches.get(2).unwrap().as_str()),
        None => (),
    };
}


fn load_file(path: &str) -> String {
    let fpath = Path::new(path);
    let mut buffer = String::new();

    match File::open(&fpath) {
        Err(x) => panic!("Error loading {}: {}", path, x.description()),
        Ok(mut x) => x.read_to_string(&mut buffer).unwrap(),
    };

    buffer
}


fn prepare_copy(line: &str) -> Option<CopyFile> {
    let capture = Regex::new(r#"^COPY +--from=([^\s]+) ("[^"]+"|[^\s"]+) +("[^"]+"|[^\s"]+)$"#).unwrap()
        .captures(line);

    match capture {
        Some(x) => {
            let stage_name = x.get(1).unwrap().as_str().to_owned();
            let from_file = x.get(2).unwrap().as_str().to_owned();
            let to_file = x.get(3).unwrap().as_str().to_owned();

            Some(CopyFile {
                stage: stage_name,
                origin_path: from_file,
                target_path: to_file,
            })
        },
        None => None,
    }
}

fn main() {
    cleanup();
    prepare_workspace();

    let dockerfile = load_file("./Dockerfile");

    polyfill_multistage(dockerfile);
}

lazy_static! {
    static ref regex_docker_command: Regex = Regex::new(r"^([A-Z]+)").unwrap();
    static ref regex_docker_copy_from: Regex = Regex::new(r#"^COPY +--from=([^\s]+) ("[^"]+"|[^\s"]+) +("[^"]+"|[^\s"]+)$"#).unwrap();
    static ref regex_docker_from: Regex = Regex::new(r"^FROM +([^\s]+) +as +([^\s]+)$").unwrap();
}

#[derive(Debug)]
struct DockerStage {
    name: String,
    steps: Vec<String>,
}

#[derive(Debug)]
struct CopyFile {
    stage: String,
    origin_path: String,
    target_path: String,
}

fn build_stages(lines: Vec<&str>) -> Vec<DockerStage> {
    let mut stages_register: Vec<DockerStage> = vec![];

    let mut lines_buffer: Vec<String> = vec![];
    let mut stage_name = "";

    for line in lines {
        if line.starts_with("FROM ") {

            // We already have some lines in our buffer, which means we need to start fresh
            if lines_buffer.len() > 0 {
                // Empty lines_buffer into the last stage
                stages_register.push(DockerStage {
                    name: stage_name.to_owned(),
                    steps: lines_buffer.clone(),
                });
                lines_buffer.clear();
            }

            let capture = regex_docker_from.captures(line);
            match capture {
                Some(matches) => {
                    stage_name = matches.get(2).unwrap().as_str(); // TODO: figure out what to do if the stage isn't named
                },
                None => (),
            };

        }

        lines_buffer.push(line.to_owned());
    }

    stages_register.push(DockerStage {
        name: stage_name.to_owned(),
        steps: lines_buffer.clone(),
    });

    stages_register
}

fn rewrite_copy(line: &str) -> String {
    match regex_docker_copy_from.captures(line) {
        Some(matches) => {
            let from_file = matches.get(2).unwrap().as_str();
            let to_file = matches.get(3).unwrap().as_str();

            format!("COPY {} {}", from_file, to_file)
        },
        None => line.to_owned()
    }
}

fn rewrite_from(line: &str) -> String {
    match regex_docker_from.captures(line) {
        Some(matches) => {
            let from_image = matches.get(1).unwrap().as_str();

            format!("FROM {}", from_image)
        },
        None => line.to_owned()
    }
}

fn polyfill_multistage(dockerfile: String) {

    let mut dockerfile_lines: Vec<&str> = dockerfile
        .split_terminator("\n")
        .map(|x| x.trim())
        .collect();
    dockerfile_lines.retain(|x| !x.is_empty());

    let mut next_lines: Vec<String> = vec![];
    let mut copy_map: BTreeMap<String, Vec<CopyFile>> = BTreeMap::new();

    for line in dockerfile_lines.clone() {
        if line.starts_with("COPY ") {
            match prepare_copy(&line) {
                Some(x) => {
                    next_lines.push(rewrite_copy(&line));
                    if ! copy_map.contains_key(&x.stage) {
                        copy_map.insert(x.stage.clone(), vec![]);
                    }
                    copy_map.get_mut(&x.stage).unwrap().push(x);
                },
                None => next_lines.push(line.to_owned())
            }
        }
        else if line.starts_with("FROM ") {
            next_lines.push(rewrite_from(&line));
        }
        else {
            next_lines.push(line.to_owned());
        }
    }

    println!("NEXTLINES:  {:?}", next_lines);


    let stages = build_stages(next_lines.iter().map(|x| &**x).collect());

    for stage in stages {
        let mut Staggerfile = File::create("./.multistagger/Staggerfile").unwrap();
        Staggerfile.write_all(&stage.steps.join("\n").into_bytes());

        let rocker_build = Rocker::build();

        println!("WOT:  {:?}", stage);

        let result = rocker_build
            .file(".multistagger/Staggerfile")
            .tag(format!("multistagger_intermediate{}", stage.name).as_str())
            .context(".")
            .init();

        println!("\n\n{:?}", result);
        panic!("\n\n\n\n");
    }

    /*
    println!("\nNEXT LINES: {:?}\n", next_lines);

    let rocker_build = Rocker::build();

    let result = rocker_build
        .file(".multistagger/Staggerfile")
        .tag("multistagger-tag")
        .context(".")
        .init();
    */
}

fn cleanup() {
    std::fs::remove_dir_all("./.multistagger");
}

fn prepare_workspace() {
    std::fs::create_dir(".multistagger");
}

/*fn unuse() {
    std::fs::create_dir(".multistagger");

    let mut split: Vec<&str> = dockerfile
        .split_terminator("\n")
        .collect();

    split.retain(|x| !x.is_empty());

    let command_reg = Regex::new(r"^([A-Z]+)").unwrap();

    let mut stages: Vec<&str> = vec![];
    let mut stages_register: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    let mut copy_register: BTreeMap<&str, Vec<String>> = BTreeMap::new();

    let mut some_copy: String = String::new();
    let mut step: String = String::new();

    println!("SPLIT: {:?}", split);

    // Just incase there is white space at the start and end of the line
    let lines: Vec<&str> = split.into_iter()
        .map(|x| x.trim())
        .filter(|x| !x.starts_with("#")) // remove comments. TODO accomodate the escaping thingo
        .collect();

    let mut next_lines: Vec<String> = vec![];

    for line in lines.clone() {
        step = match command_reg.captures(line).unwrap().get(1).unwrap().as_str() {
            "FROM" => {
                init_stage(&mut stages, line);
                stages_register.insert(stages.last().unwrap(), vec![]);
                line.to_owned()
            },
            "COPY" => {
                some_copy = prep_copy(&mut stages_register, &mut copy_register, line);
                println!("PIRINTSOM: {}", some_copy);
                some_copy
            }
            _ => line.to_owned()
        };
        stages_register.get_mut(stages.last().unwrap()).unwrap().push(step.to_owned());
        next_lines.push(step);
    }

    let mut args: Vec<String> = env::args().collect();

    args.remove(0);

    let mut docker_file_buffer = vec![];
    let mut docker_id: String;
    let docker_args = vec!["build".to_owned(), "-f".to_owned(), "./.multistagger/Staggerfile".to_owned(), ".".to_owned()];
    let lines_len = next_lines.len();

    println!("LINES:  {:?}", next_lines);


    for (index, line) in next_lines.into_iter().enumerate() {
        if docker_file_buffer.len() > 0 {
            if line.starts_with("FROM") || lines_len == index + 1 {
                let mut Dockerfile = File::create("./.multistagger/Staggerfile").unwrap();
                Dockerfile.write_all(&docker_file_buffer.join("\n").into_bytes());

                docker_file_buffer.clear();

                docker_id = docker(&docker_args);
            }
        }

        println!("LENGHT: {}, {}", lines_len, docker_file_buffer.len());

        if index < lines_len - 1 {
            docker_file_buffer.push(line);
        }
    }

    println!("{:?} \n\n {:?} \n\n {:?}", stages_register, copy_register, args);
}

*/
