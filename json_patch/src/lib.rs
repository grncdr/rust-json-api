#![feature(slice_splits)]
extern crate serde_json;

mod patch;

use serde_json::Value;
use std::error::Error;
use std::fmt;

pub use patch::{apply, PatchError};

pub struct Patch {
    pub ops: Vec<Op>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Op {
    Add(Path, Value),
    Remove(Path),
    Replace(Path, Value),
    Copy(Path, Path),
    Move(Path, Path),
    Test(Path, Value),
}

pub type Path = Vec<String>;

#[derive(Debug)]
pub enum InvalidPatchError {
    JsonError(serde_json::Error),
    MustBeArray,
    BadOp(usize, InvalidOpError),
}

#[derive(Debug)]
pub enum InvalidOpError {
    UnknownOp(String),
    MissingProperty(String),
    MustBeString(String),
}

impl From<serde_json::Error> for InvalidPatchError {
    fn from(e: serde_json::Error) -> InvalidPatchError {
        InvalidPatchError::JsonError(e)
    }
}

impl Patch {
    pub fn from_str(s: &str) -> Result<Patch, InvalidPatchError> {
        Patch::from_value(try!(serde_json::from_str(s)))
    }

    pub fn from_value(v: Value) -> Result<Patch, InvalidPatchError> {
        match v {
            Value::Array(values) => {
                let mut ops: Vec<Op> = Vec::with_capacity(values.len());
                for (i, value) in values.into_iter().enumerate() {
                    match Op::from_value(value) {
                        Ok(op) => ops.push(op),
                        Err(e) => return Err(InvalidPatchError::BadOp(i, e)),
                    }
                }
                Ok(Patch { ops: ops })
            }
            _ => Err(InvalidPatchError::MustBeArray),
        }
    }
}

impl Op {
    pub fn from_value(v: Value) -> Result<Op, InvalidOpError> {
        let (op, path, from) = {
            let op = try!(require_key_as_string(&v, "op")).to_string();
            let path = try!(require_key_as_path(&v, "path"));
            let from = require_key_as_path(&v, "from");  // optional, we fail lower
            (op, path, from)
        };
        Ok(match op.as_ref() {
            "add" => Op::Add(path, try!(move_value(v))),
            "remove" => Op::Remove(path),
            "replace" => Op::Replace(path, try!(move_value(v))),
            "test" => Op::Test(path, try!(move_value(v))),
            "copy" => Op::Copy(path, try!(from)),
            "move" => Op::Move(path, try!(from)),
            _ => return Err(InvalidOpError::UnknownOp(op)),
        })
    }
}

impl fmt::Display for Patch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.ops)
    }
}

impl fmt::Display for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Op::Add(ref path, ref value) => {
                write!(f,
                       r#"{{"op":"add","path":{:?},"value":{:?}}}"#,
                       path,
                       value)
            }
            &Op::Remove(ref path) => {
                write!(f, r#"{{"op":"remove","path":{:?}}}"#, path)
            }
            &Op::Replace(ref path, ref v) => {
                write!(f,
                       r#"{{"op":"replace","path":{:?},"value":{:?}}}"#,
                       path,
                       v)
            }
            &Op::Copy(ref path, ref to) => {
                write!(f, r#"{{"op":"copy","path":{:?},"to":{:?}}}"#, path, to)
            }
            &Op::Move(ref path, ref to) => {
                write!(f, r#"{{"op":"move","path":{:?},"to":{:?}}}"#, path, to)
            }
            &Op::Test(ref path, ref v) => {
                write!(f,
                       r#"{{"op":"move","path":{:?},"value":{:?}}}"#,
                       path,
                       v)
            }
        }
    }
}
fn move_value(mut v: Value) -> Result<Value, InvalidOpError> {
    let mut o = v.as_object_mut().unwrap(); // don't panic, we only get here if the thing was already an object
    match o.remove("value") {
        Some(v) => Ok(v),
        None => Err(InvalidOpError::MissingProperty("value".to_string())),
    }
}

fn require_key<'a>(v: &'a Value, k: &str) -> Result<&'a Value, InvalidOpError> {
    match v.find(k) {
        None => Err(InvalidOpError::MissingProperty(k.to_string())),
        Some(v) => Ok(v),
    }
}

fn require_key_as_string<'a>(v: &'a Value, k: &str) -> Result<&'a str, InvalidOpError> {
    match require_key(v, k) {
        Ok(v) => match v.as_string() {
            Some(s) => Ok(s),
            None => Err(InvalidOpError::MustBeString(k.to_string())),
        },
        Err(error) => Err(error),
    }
}

fn require_key_as_path<'a>(v: &'a Value, k: &str) -> Result<Path, InvalidOpError> {
    require_key_as_string(v, k).map(|s| s.split("/").skip(1).map(|s| s.to_string()).collect())
}
