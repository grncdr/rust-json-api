extern crate hyper;
#[macro_use]
extern crate mime;
extern crate unicase;
extern crate serde_json;
extern crate json_patch;


#[macro_use]
mod macros;
pub mod server;
mod database;
mod shared_value;
mod patch_helpers;

pub fn main() {
    server::start("0.0.0.0:3000");
}
