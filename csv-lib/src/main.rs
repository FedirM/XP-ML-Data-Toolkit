use csv_lib::error::Result;
use csv_lib::CsvToolkit;

use std::path::Path;

fn main() -> Result<()> {
    let mut toolkit =
        CsvToolkit::new(Path::new("./tests/test.csv"), b',', None, false, None, None)?;
    let test_key = String::from("Exercise Hours Per Week");

    toolkit.normalizing(vec![test_key.clone()])?;
    Ok(())
}
