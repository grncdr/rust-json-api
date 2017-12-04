use std::io;
use std::io::prelude::*;
use std::collections::HashMap;
use std::fs::{File, create_dir_all, OpenOptions};
use std::path::Path;
use std::sync::{RwLock, PoisonError};
use json_patch::{apply, Op, Patch, InvalidPatchError, PatchError};
use serde_json::Value;

use shared_value::SharedValue;
use patch_helpers::prefix_patch_paths;

/// A `Doc` wraps a shared value and writes all successfully applied patches to an owned `Write`
pub struct Doc<W: Write> {
    value: SharedValue,
    version: usize,
    writer: W,
}

pub struct Database<W: Write> {
    dir: String,
    docs: RwLock<HashMap<String, Doc<W>>>,
}

#[derive(Debug)]
pub enum DbError {
    IoError(io::Error),
    PatchError(PatchError),
    InvalidPatchError(InvalidPatchError),
    DocumentDoesNotExist,
    PathDoesNotExist,
    PoisonError,
}

wrap_error!(InvalidPatchError, DbError::InvalidPatchError);
wrap_error!(PatchError, DbError::PatchError);
wrap_error!(io::Error, DbError::IoError);

impl <T> From<PoisonError<T>> for DbError {
    fn from(_: PoisonError<T>) -> DbError {
        DbError::PoisonError
    }
}

impl Database<File> {
    pub fn open(dir: &str) -> Result<Database<File>, io::Error> {
        try!(create_dir_all(Path::new(dir)));
        Ok(Database {
            dir: dir.to_string(),
            docs: RwLock::new(HashMap::new()),
        })
    }

    pub fn find_in_doc(&self, id: &str, path: &[&str]) -> Result<Value, DbError> {
        let live_docs = try!(self.docs.read());
        if let Some(doc) = live_docs.get(id) {
            doc.value.clone_path(path).ok_or(DbError::PathDoesNotExist)
        } else {
            Err(DbError::DocumentDoesNotExist)
        }
    }

    pub fn patch_doc(&self, id: &str, patch: Patch, prefix: &[&str]) -> Result<Value, DbError> {
        let mut live_docs = try!(self.docs.write());
        if !live_docs.contains_key(id) {
            live_docs.insert(id.to_string(),
                             try!(self.load(id, prefix.len() > 0)));
        }

        let doc = live_docs.get_mut(id).unwrap();
        let scoped_patch = prefix_patch_paths(prefix, patch);

        try!(doc.value.patch(&scoped_patch));
        try!(writeln!(doc.writer, "{}", scoped_patch));

        doc.value.clone_path(prefix).ok_or(DbError::PathDoesNotExist)
    }

    fn open_doc(&self, id: &str, must_exist: bool) -> Result<&Doc<File>, DbError> {
        {
            // scope our read lock so that self.load can succeed
            let live_docs = try!(self.docs.read());
            if let Some(ref doc) = live_docs.get(id) {
                return Ok(doc)
            }
        }
        match self.load(id, must_exist) {
            Ok(ref doc) => Ok(doc),
            Err(err) => Err(err),
        }
    }

    fn load(&self, id: &str, must_exist: bool) -> Result<Doc<File>, DbError> {
        let filename = Path::new(&self.dir).join(id);
        let reader = io::BufReader::new(try!(OpenOptions::new()
                                                 .read(true)
                                                 .create(must_exist)
                                                 .open(&filename)));

        let mut version: usize = 0;
        let mut value = Value::Null;

        for maybe_line in reader.lines() {
            let line = try!(maybe_line);
            let patch = try!(Patch::from_str(&line));
            value = try!(apply(&patch, &value));
            version += 1;
        }

        let writer = try!(OpenOptions::new()
                              .write(true)
                              .create(false)
                              .append(true)
                              .truncate(false)
                              .open(&filename));

        Ok(Doc {
            value: SharedValue::from_value(value),
            version: version,
            writer: writer,
        })
    }
}
