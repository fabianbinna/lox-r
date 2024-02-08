use std::{fmt::Display, fs::read_to_string, io::{self, Write}, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

use crate::object::Object;

#[derive(Clone)]
pub enum NativeFunction {
    Clock,
    Input,
    ReadFile
}

impl NativeFunction {

    pub fn call(&self, args: Vec<Object>) -> Object {
        match self {
            NativeFunction::Clock => self.clock(),
            NativeFunction::Input => self.input(),
            NativeFunction::ReadFile => {
                match args.first() {
                    Some(obj) => {
                        if let Object::String(path) = obj {
                            self.read_file(path.clone())
                        } else {
                            panic!("Native function [read_file]: Expecting string as path argument.")
                        }
                    },
                    None => panic!("Native function [read_file]: Expecting argument [path].")
                }
            },
        }
    }

    fn clock(&self) -> Object {
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos() as f64;
        Object::Number(time)
    }

    fn input(&self) -> Object {
        let mut input = String::new();
        let _ = io::stdout().flush();
        match io::stdin().read_line(&mut input) {
            Ok(_) => Object::String(input),
            Err(_) => Object::Nil
        }
    }

    fn read_file(&self, path: String) -> Object {
        println!("{}", path.clone());
        let path_buf = PathBuf::from(path.trim());
        
        match read_to_string(path_buf) {
            Ok(data) => Object::String(data),
            Err(e) => panic!("{}", e)
        }
    }
}

impl Display for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NativeFunction::Clock => write!(f, "{}", "clock"),
            NativeFunction::Input => write!(f, "{}", "input"),
            NativeFunction::ReadFile => write!(f, "{}", "readFile"),
        }
    }
}