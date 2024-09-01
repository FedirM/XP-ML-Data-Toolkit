use csv_lib::deserialization::generate_struct;
use std::path::Path;

#[test]
fn test_struct_generation() {
    let source = Path::new("./tests/test.csv");
    let dist = Path::new("./tests/out/test.rs");

    match generate_struct(source, dist) {
        Ok(res) => println!("{res}"),
        Err(e) => panic!("{e}"),
    }
}
