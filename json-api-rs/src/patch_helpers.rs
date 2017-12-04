use json_patch::{Patch, Op};

macro_rules! concat {
    ($prefix:ident, $path:ident) => {{
        let mut result: Vec<String> = $prefix.iter().map(|s| s.to_string()).collect();
        result.extend($path);
        result
    }}
}

pub fn prefix_patch_paths(prefix: &[&str], patch: Patch) -> Patch {
    let ops = patch.ops.into_iter().map(move |op| {
        match op {
            Op::Add(path, value) => Op::Add(concat!(prefix, path), value),
            Op::Remove(path) => Op::Remove(concat!(prefix, path)),
            Op::Replace(path, value) => Op::Replace(concat!(prefix, path), value),
            Op::Copy(path, from) => Op::Copy(concat!(prefix, path), from),
            Op::Move(path, from) => Op::Move(concat!(prefix, path), from),
            Op::Test(path, value) => Op::Test(concat!(prefix, path), value),
        }
    });
    Patch { ops: ops.collect() }
}
