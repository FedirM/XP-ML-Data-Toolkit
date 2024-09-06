use std::time::{SystemTime, UNIX_EPOCH};

pub const IMPORTS: [&str; 2] = ["use csv::Reader;", "use std::path::Path;"];
pub const STRUCT_DERIVE: &str = "#[derive(Debug, Clone, PartialEq, Eq)]";

pub fn generate_temporary_file_name() -> String {
    let tstmp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    println!("Timestamp: {}", tstmp.as_millis());

    return format!("temporary_{}_.dat", tstmp.as_millis());
}
