use std::io;

//use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::RwLock;

//use patch_helpers::prefix_patch_paths;
use serde_json::Value;
use json_patch::{apply, Patch, InvalidPatchError, PatchError};

/// Thread-safe Wrapper around a serde_json::Value
#[derive(Debug)]
pub struct SharedValue {
    value: RwLock<Value>,
}

impl SharedValue {

    #[inline]
    pub fn from_value(value: Value) -> SharedValue {
        SharedValue { value: RwLock::new(value) }
    }

    /// Apply patches to the underlying value in a threadsafe way
    pub fn patch(&self, patch: &Patch) -> Result<(), PatchError> {
        let mut value = self.value.write().unwrap();
        let next = try!(apply(patch, &value));
        *value = next;
        Ok(())
    }

/* Lock-free impl
    /// Apply patches to the underlying value in a threadsafe way w/o locking
    fn patch(&self, patch: &Patch) -> Result<(), PatchError> {
        loop {
            let prev = self.value.load(Ordering::Relaxed);
            let prevv = unsafe { *prev };
            let next = try!(apply(patch, &prevv));
            // try!(write_patch(patch));
            if prev == self.value.compare_and_swap(prev, &mut next, Ordering::Relaxed) {
                self.version.fetch_add(1, Ordering::SeqCst);
                break;
            }
        }
        Ok(())
    }
*/

    pub fn clone_path(&self, path: &[&str]) -> Option<Value> {
        let value = self.value.read().unwrap();
        (*value).find_path(path).cloned()
    }
}
