#[macro_use]
extern crate nickel;
extern crate hyper;
extern crate serde_json;
extern crate json_patch;

use nickel::Nickel;

use std::sync::RwLock;
use std::io::Read;
use json_patch::{Patch, InvalidPatch, apply};

#[derive(Debug)]
pub enum AppError {
    PatchFailed(InvalidPatch),
}

impl From<AppError> for nickel::NickelError {
    fn from(err: AppError) -> nickel::NickelError {
        match AppError {
            PatchFailed(err) => format!("{:?}"
        AppError::PatchFailed(err)
    }
}

fn main() {
    let lock: RwLock<serde_json::Value> = RwLock::new(serde_json::from_str("{}").unwrap());
    let mut server = Nickel::new();

    server.utilize(router! {
        get "**" => |_req, _res| {
            "Hello world!"
        }
        patch "**" => |req, _res| {
            let mut b = String::new();
            req.origin.read_to_string(&mut b).unwrap();
            let patch = try!(Patch::from_str(&b).map_err(|e| res.error(400, format!(e))));
            format!("{:?}", patch.ops)
        }
    });

    server.listen("127.0.0.1:6767");
}

fn prefix_paths() {
}
