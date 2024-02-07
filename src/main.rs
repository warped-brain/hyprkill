use serde_json::{Error, Value};
use std::{
    collections::HashMap,
    env,
    process::{self, Output},
};
use sysinfo::{Pid, Process, ProcessExt, System, SystemExt};

fn main() {
    let input: Vec<String> = env::args().skip(1).collect();
    let mut hmap = input_parser(input);
    hyprctl_parser(&mut hmap);

    process_killer(&hmap)
}

#[derive(Debug)]
enum Errors {
    NoProcessFound,
    FailedToKill,
    HprctlError(String),
    SerdeParseError(serde_json::Error),
}

fn input_parser(input: Vec<String>) -> HashMap<String, Option<usize>> {
    let mut hmap: HashMap<String, Option<usize>> = HashMap::new();
    for titles in input {
        hmap.insert(titles, None);
    }
    hmap
}

struct workspace {
    id: i8,
    name: String,
}
struct window_properties {
    address: String,
    mapped: bool,
    hidden: bool,
    at: Vec<i8>,
    size: Vec<i8>,
    workspace: workspace,
    floating: bool,
    monitor: i8,
    class: String,
    title: String,
    initialClass: String,
    initialTitle: String,
    pid: usize,
    xwayland: bool,
    pinned: bool,
    fullscreen: bool,
    fullscreenMode: i8,
    fakeFullscreen: bool,
    grouped: Vec<i8>,
    swallowing: String,
}
fn cmd_out_json_parser(mut inp: Vec<u8>) -> Result<Value, Errors> {
    let x = serde_json::from_slice(&inp[..]);
    if let Ok(val) = x {
        return Ok(val);
    } else {
        Err(Errors::SerdeParseError(x.err().unwrap()))
    }
}

fn hyprctl_parser(hmap: &mut HashMap<String, Option<usize>>) -> Result<(), Errors> {
    let cmd_result = process::Command::new("hyprctl")
        .arg("clients")
        .arg("-j")
        .output();
    if let Ok(output) = cmd_result {
        let parsed_json = cmd_out_json_parser(output.stdout);
        if let Ok(json_objects) = parsed_json {
            for i in 0..json_objects.as_array().unwrap().len() {
                let pid: usize = usize::from_str_radix(
                    json_objects[i].get("pid").unwrap().to_string().as_str(),
                    10,
                )
                .unwrap();
                let title = json_objects[i]
                    .get("title")
                    .unwrap()
                    .to_string()
                    .strip_prefix("\"")
                    .unwrap()
                    .strip_suffix("\"")
                    .unwrap()
                    .to_string();
                if hmap.contains_key(&title) {
                    hmap.insert(title, {
                        if pid == 0 {
                            None
                        } else {
                            Some(pid)
                        }
                    });
                }
            }
        } else {
            return Err(parsed_json.err().unwrap());
        }
    } else {
        return Err(Errors::HprctlError(cmd_result.err().unwrap().to_string()));
    }
    Ok(())
}

fn process_killer(inputs: &HashMap<String, Option<usize>>) {
    for i in inputs {
        match i.1 {
            None => println!("\nUnknown window title: {}", i.0),
            Some(x) => {
                if let Ok(v) = kill_process(x.to_owned()) {
                    println!("\nKilled window {} with pid: {}", i.0, x);
                }
            }
        }
    }
}

fn kill_process(pid: usize) -> Result<bool, Errors> {
    let s = System::new_all();
    let mut process: Option<&Process>;
    // for pid in pids {
    process = s.process(Pid::from(pid));
    match process {
        Some(process) => process.kill(),
        None => return Err(Errors::FailedToKill),
    };
    // }
    Ok(true)
}
