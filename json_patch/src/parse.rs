#![feature(slice_patterns)]
#![feature(slice_splits)]
extern crate serde_json;

use serde_json::Value;

#[derive(Debug)]
pub enum Op {
    Add(Path, Value),
    Remove(Path),
    Replace(Path, Value),
    Copy(Path, Path),
    Move(Path, Path),
    Test(Path, Value),
}

pub type Path = Vec<String>;

impl Op {
    pub fn from_value(v: Value) -> Result<Op, ParseError> {
        let op = try!(require_key_as_string(v, "op"));
        let path = try!(require_key_as_path(v, "path"));
        match op.as_ref() {
            "add" => Ok(Op::Add(path, try!(require_key(v, "value")))),
            "remove" => Ok(Op::Remove(path)),
            "replace" => Ok(Op::Replace(path, try!(require_key(v, "value")))),
            "test" => Ok(Op::Test(path, try!(require_key(v, "value")))),
            "copy" => Ok(Op::Copy(path, try!(require_key_as_path(v, "from")))),
            "move" => Ok(Op::Move(path, try!(require_key_as_path(v, "from")))),
            _ => Err(ParseError::Schema(format!("invalid op \"{}\"", op).to_string())),
        }
    }
}

fn require_key(v: &Value, k: &str) -> Result<&Value, ParseError> {
    v.find(k).ok_or(ParseError::Schema(format!("\"{}\" is required", k)))
}

fn require_key_as_string(v: &Value, k: &str) -> Result<String, ParseError> {
    require_key(v, k).and_then(|v| {
        v.as_string()
         .ok_or(ParseError::Schema(format!("\"{}\" property must be a string".to_string(), k)))
    })
}

fn require_key_as_path(v: &Value, k: &str) -> Result<Path, ParseError> {
    require_key_as_string(v, k).map(|s| s.split("/").skip(1).map(|s| s.to_string()).collect())
}
