use Op;
use Patch;

use serde_json::Value;

#[derive(Debug,PartialEq)]
pub struct PatchError;

pub fn apply(patch: &Patch, v: &Value) -> Result<Value, PatchError> {
    let mut v2 = v.clone();
    for op in &patch.ops {
        try!(apply_op(op, &mut v2))
    }
    Ok(v2)
}

pub fn apply_op(op: &Op, root: &mut Value) -> Result<(), PatchError> {
    match op {
        &Op::Add(ref path, ref value) => {
            if path.len() == 0 {
                *root = value.clone();
                return Ok(())
            }
            path.split_last().ok_or(PatchError).and_then(|(key, path)| {
                get_path(root, path)
                    .ok_or(PatchError)
                    .and_then(|parent| insert_key(parent, key, value.clone()))
            })
        }
        &Op::Replace(ref path, ref value) => {
            if path.len() == 0 {
                *root = value.clone();
                return Ok(())
            }
            path.split_last().ok_or(PatchError).and_then(|(key, parent_path)| {
                get_path(root, parent_path)
                    .ok_or(PatchError)
                    .and_then(|parent| replace_key(parent, key, value.clone()))
            })
        }
        &Op::Remove(ref path) => {
            if path.len() == 0 {
                return Err(PatchError)
            }
            path.split_last().ok_or(PatchError).and_then(|(key, parent_path)| {
                get_path(root, parent_path)
                    .ok_or(PatchError)
                    .and_then(|parent| remove_key(parent, key).map(|_| ()))
            })
        }
        &Op::Test(ref path, ref test_value) => {
            get_path(root, &path)
                .ok_or(PatchError)
                .and_then(|current_value| {
                    if &current_value.clone() == test_value {
                        Ok(())
                    } else {
                        Err(PatchError)
                    }
                })
        }

        &Op::Move(ref to, ref from) => {
            let value = try!(from.split_last().ok_or(PatchError).and_then(|(key, path)| {
                get_path(root, path)
                    .ok_or(PatchError)
                    .and_then(|parent| remove_key(parent, key))
            }));

            to.split_last().ok_or(PatchError).and_then(|(key, path)| {
                get_path(root, path)
                    .ok_or(PatchError)
                    .and_then(|parent| insert_key(parent, key, value.clone()))
            })
        }

        &Op::Copy(ref to, ref from) => {
            let from_path: Vec<&str> = from.iter().map(|s| &s[..]).collect();
            let value = try!(root.find_path(&from_path)
                                 .ok_or(PatchError)
                                 .map(|v| v.clone()));

            to.split_last().ok_or(PatchError).and_then(|(dest_key, dest_path)| {
                get_path(root, dest_path)
                    .ok_or(PatchError)
                    .and_then(|parent| insert_key(parent, dest_key, value))
            })
        }
    }
}

fn get_key<'a>(c: &'a mut Value, key: &str) -> Option<&'a mut Value> {
    match c {
        &mut Value::Object(ref mut o) => o.get_mut(key),
        &mut Value::Array(ref mut a) => {
            match string_to_index(key, a.len()).ok() {
                Some(i) => a.get_mut(i),
                None => return None,
            }
        }
        _ => return None,
    }
}

fn get_path<'a>(root: &'a mut Value, keys: &[String]) -> Option<&'a mut Value> {
    match keys.split_first() {
        None => Some(root),
        Some((first, rest)) => {
            match get_key(root, first) {
                None => None,
                Some(child) => get_path(child, rest),
            }
        }
    }
}

fn insert_key(container: &mut Value, key: &str, value: Value) -> Result<(), PatchError> {
    match container {
        &mut Value::Object(ref mut o) => {
            o.insert(key.to_string(), value);
            Ok(())
        }
        &mut Value::Array(ref mut a) => {
            let i = try!(string_to_index(&key, a.len() + 1));
            if i == a.len() {
                a.push(value);
            } else {
                a.insert(i, value);
            }
            Ok(())
        }
        _ => Err(PatchError),
    }
}

fn replace_key(container: &mut Value, key: &str, value: Value) -> Result<(), PatchError> {
    match container {
        &mut Value::Object(ref mut o) => {
            o.insert(key.to_string(), value);
            Ok(())
        }
        &mut Value::Array(ref mut a) => {
            let i = try!(string_to_index(&key, a.len() + 1));
            a[i] = value;
            Ok(())
        }
        _ => Err(PatchError),
    }
}

fn remove_key(container: &mut Value, key: &str) -> Result<Value, PatchError> {
    let v: Option<Value> = match container {
        &mut Value::Object(ref mut o) => {
            o.remove(key)
        }
        &mut Value::Array(ref mut a) => {
            let i = try!(string_to_index(key, a.len() + 1));
            Some(a.remove(i))
        }
        _ => None,
    };
    v.ok_or(PatchError)
}

fn string_to_index(k: &str, size: usize) -> Result<usize, PatchError> {
    if k == "-" {
        Ok(size)
    } else {
        match k.parse::<usize>() {
            Ok(i) if i < size => Ok(i),
            _ => Err(PatchError),
        }
    }
}

macro_rules! apply_patch {
    ($doc_str:expr, $patch_str:expr) => {{
        use serde_json;
        let mut root: serde_json::Value = serde_json::from_str($doc_str).unwrap();
        let patch = Patch::from_str($patch_str).unwrap();
        apply(&patch, &mut root).unwrap();
        root
    }}
}

#[test]
fn add_in_object() {
    let root = apply_patch!("{}", r#"[{"op":"add","path":"/hi","value":12}]"#);
    assert_eq!(root.find("hi").unwrap().as_u64().unwrap(), 12);
}

#[test]
fn add_nested_value() {
    let root = apply_patch!(r#"{"o":{}}"#,
                            r#"[{"op":"add","path":"/o/cool", "value":"beans"}]"#);
    assert_eq!(root.lookup("o.cool").unwrap().as_string().unwrap(),
               "beans".to_string())
}

#[test]
fn add_and_remove() {
    let root = apply_patch!("{}",
                            r#"[
                            {"op":"add","path":"/one", "value":1},
                            {"op":"add","path":"/two", "value":2},
                            {"op":"remove","path":"/one"}
                            ]"#);
    assert_eq!(root.find("one"), None);
    assert_eq!(root.find("two").unwrap().as_u64(), Some(2))
}

#[test]
#[should_panic(expected = "PatchError")]
fn add_fails_without_parent() {
    let _ = apply_patch!("null",
                         r#"[{"op":"add", "path": "/yo", "value":1}]"#);
}

#[test]
fn add_with_empty_path_replaces_root() {
    let root = apply_patch!("null", r#"[{"op":"add","path":"","value":12}]"#);
    assert_eq!(root.as_u64().unwrap(), 12)
}
