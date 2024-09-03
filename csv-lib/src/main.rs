// use csv_lib::deserialization::generate_struct;
use csv_lib::error::Result;
use csv_lib::CsvToolkit;

use csv_lib::deserialization::{parse_col_type, DeserializationType};

fn main() -> Result<()> {

    let raw = vec!["3", "3.3", "kek lol chebureck!", "FALSE", "  "];
    let types = raw
        .into_iter()
        .map(|s| parse_col_type(s).unwrap())
        .collect::<Vec<DeserializationType>>();

    println!("V: {:#?}", types);

    // let mut tool = CsvToolkit::new(
    //     std::path::Path::new("./tests/test.csv"),
    //     b',',
    //     None,
    //     false,
    //     None,
    //     None
    // )?;

    // println!("Min: {:?}", tool.min);
    // println!("Max: {:?}", tool.max);

    // // println!("\n\n{}", generate_struct(
    // //     std::path::Path::new("./tests/test.csv"), 
    // //     std::path::Path::new("./tests/test.rs")
    // // )?);

    // tool.simple_iter()?;

    Ok(())
}