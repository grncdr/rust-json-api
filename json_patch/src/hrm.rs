extern crate serde_json;

#[test]
fn update_a_nested_thing() {
    let root: serde_json::Value = serde_json::from_str(r#"{"a":{"b":{"c":"d"}}}"#).unwrap();
    println!("{:?}", root);
    let b = root.as_object_mut().unwrap().get("a").unwrap();
    //println!("{:?}", b.get("c").unwrap())
}
