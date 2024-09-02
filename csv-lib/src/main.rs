use csv_lib::deserialization::generate_struct;
use csv_lib::error::Result;
use csv_lib::CsvToolkit;

fn main() -> Result<()> {

    let tool = CsvToolkit::new(
        std::path::Path::new("./tests/test.csv"),
        b',',
        None,
        false,
        None,
        None
    )?;

    println!("TRIM: {:?}", String::from("   ").trim());

    println!("Min: {:?}", tool.min);
    println!("Max: {:?}", tool.max);

    println!("\n\n{}", generate_struct(
        std::path::Path::new("./tests/test.csv"), 
        std::path::Path::new("./tests/test.txt")
    )?);

    Ok(())
}