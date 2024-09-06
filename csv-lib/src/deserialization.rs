use std::{
    cmp::min,
    fs::{self, File},
    io::Write,
    ops::Sub,
    path::Path,
};

use csv::Reader;
use regex::Regex;

use crate::constants::{IMPORTS, STRUCT_DERIVE};
use crate::error::Result;

#[macro_export]
macro_rules! min {
    ($x:expr) => {
        match $x {
            DeserializationType::NUMBER(_) => $x,
            _ => panic!("Incompatible expression type!"),
        }
    };

    ($x:expr, $y:expr) => {
        match ($x, $y) {
            (DeserializationType::NUMBER(a), DeserializationType::NUMBER(b)) => {
                if a < b { DeserializationType::NUMBER(a) } else { DeserializationType::NUMBER(b) }
            },
            _ => panic!("Incompatible expression type!"),
        }
    };

    ($x:expr, $y:expr, $( $rest:expr ),+) => {
        {
            match ($x, $y) {
                (DeserializationType::NUMBER(a), DeserializationType::NUMBER(b)) => {
                    let min_of_two = if a < b { DeserializationType::NUMBER(a) } else { DeserializationType::NUMBER(b) };
                    min!(min_of_two, $( $rest ),+)
                },
                _ => panic!("Incompatible expression type!"),
            }
        }
    };
}

#[macro_export]
macro_rules! max {
    ($x:expr) => {
        match $x {
            DeserializationType::NUMBER(_) => $x,
            _ => panic!("Incompatible expression type!"),
        }
    };

    ($x:expr, $y:expr) => {
        match ($x, $y) {
            (DeserializationType::NUMBER(a), DeserializationType::NUMBER(b)) => {
                if a > b { DeserializationType::NUMBER(a) } else { DeserializationType::NUMBER(b) }
            },
            _ => panic!("Incompatible expression type!"),
        }
    };

    ($x:expr, $y:expr, $( $rest:expr ),+) => {
        {
            match ($x, $y) {
                (DeserializationType::NUMBER(a), DeserializationType::NUMBER(b)) => {
                    let max_of_two = if a > b { DeserializationType::NUMBER(a) } else { DeserializationType::NUMBER(b) };
                    max!(max_of_two, $( $rest ),+)
                },
                _ => panic!("Incompatible expression type!"),
            }
        }
    };
}

#[derive(Debug, Clone)]
pub enum DeserializationType {
    NUMBER(f64),
    BOOLEAN(bool),
    STRING(String),
    EMPTY,
}

impl Sub<Self> for DeserializationType {
    type Output = f64;

    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (DeserializationType::NUMBER(x), DeserializationType::NUMBER(y)) => x.sub(y),
            _ => panic!("Could not subtract non numeric value(s)."),
        }
    }
}

impl PartialEq for DeserializationType {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (DeserializationType::BOOLEAN(x), DeserializationType::BOOLEAN(y)) => x.eq(y),
            (DeserializationType::NUMBER(x), DeserializationType::NUMBER(y)) => x.eq(y),
            (DeserializationType::STRING(x), DeserializationType::STRING(y)) => x.eq(y),
            (DeserializationType::EMPTY, DeserializationType::EMPTY) => true,
            _ => false,
        }
    }
}

impl PartialOrd for DeserializationType {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (DeserializationType::NUMBER(x), DeserializationType::NUMBER(y)) => x.partial_cmp(y),
            _ => None,
        }
    }
}

impl DeserializationType {
    pub fn display(&self) -> String {
        format!("{}", self)
    }

    pub fn is_same_type(&self, other: &DeserializationType) -> bool {
        match (self, other) {
            (DeserializationType::BOOLEAN(_), DeserializationType::BOOLEAN(_)) => true,
            (DeserializationType::NUMBER(_), DeserializationType::NUMBER(_)) => true,
            (DeserializationType::STRING(_), DeserializationType::STRING(_)) => true,
            (DeserializationType::EMPTY, DeserializationType::EMPTY) => true,
            _ => false,
        }
    }

    pub fn is_ordered(&self) -> bool {
        return self.is_same_type(&DeserializationType::NUMBER(f64::default()));
    }
}

impl std::fmt::Display for DeserializationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_type = match self {
            DeserializationType::NUMBER(_) => "f64".to_owned(),
            DeserializationType::BOOLEAN(_) => "bool".to_owned(),
            DeserializationType::STRING(_) => "String".to_owned(),
            DeserializationType::EMPTY => "String".to_owned(),
        };

        write!(f, "{col_type}")
    }
}

pub fn parse_col_type(value: &str) -> Result<DeserializationType> {
    let int_re = Regex::new(r"^\d+$").unwrap();
    let float_re = Regex::new(r"^(\d+)?\.\d+$").unwrap();

    let value = value.trim();
    match value {
        _ if int_re.is_match(value) | float_re.is_match(value) => {
            Ok(DeserializationType::NUMBER(value.parse()?))
        }
        "FALSE" | "False" | "false" | "TRUE" | "True" | "true" => {
            Ok(DeserializationType::BOOLEAN(value.to_lowercase().parse()?))
        }
        _ if !value.is_empty() => Ok(DeserializationType::STRING(value.to_owned())),
        _ => Ok(DeserializationType::EMPTY),
    }
}

/// Make a mod file to work with passed csv source file in Rust
///
/// This function takes a path to the source csv file, analyze it and return a mod in a String format to work with those data in common Rust format.
/// Could be handy as a part of CLI tool.
///
/// # Arguments
/// `source` - Path to source csv file
/// `dist` - Path to the output dir/file
///
/// # Return
/// A copy of `dist` content as a String
///
/// # Errors
/// fs, io, type coersion;

pub fn generate_struct<F: AsRef<Path>>(source: F, dist: F) -> Result<String> {
    let mut reader = Reader::from_path(source.as_ref())?;
    let mut headers: Vec<String> = reader
        .headers()?
        .clone()
        .into_iter()
        .map(|s| s.trim().to_owned())
        .collect();
    headers = parse_headers(headers);

    let mut data_row: Vec<String> = vec![String::default(); headers.len()];
    let mut tmp: Vec<String>;

    for str_result in reader.records() {
        let str_rec = str_result?;
        tmp = str_rec.into_iter().map(|s| s.trim().to_owned()).collect();
        println!("Curr row: {:?}", tmp);

        // Prove: both vec assigned above!
        for i in 0..min(data_row.len(), tmp.len()) {
            if data_row[i].is_empty() && !tmp[i].is_empty() {
                data_row[i] = tmp[i].clone();
            }
        }

        if !data_row.contains(&String::default()) {
            println!("Break {:?}", &data_row);
            break;
        }
    }

    let struct_name = if let Some(stem) = source.as_ref().file_stem().take() {
        to_struct_name(stem.to_str().unwrap())
    } else {
        to_struct_name("CsvData")
    };

    let mut struct_tmpl = format!("{STRUCT_DERIVE}\npub struct {}{{", &struct_name);

    let mut from_impl = format!(
        "impl From<Vec<String>> for {}{{\n\tfn from(value: Vec<String>) -> Self {{\n\t\tSelf{{",
        &struct_name
    );

    for (id, h_item) in headers.iter().enumerate() {
        let value_type = parse_col_type(&data_row[id])?;
        struct_tmpl += &format!("\n\t{h_item}: {},", value_type.display());
        from_impl += &format!("\n\t\t\t{h_item}: value[{id}].parse().expect(&format!(\"Could not parse '{{}}' for '{h_item}'!\", value[{id}])),");
    }

    struct_tmpl += "\n}";
    from_impl += "\n\t\t}\n\t}\n}";

    if let Some(parent) = dist.as_ref().to_path_buf().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let module = format!(
        r#"// Imports
{}

// struct definition
{}

// Implement struct coersion from raw data strings
{}

// Initial functions
pub fn init(source: impl AsRef<Path>) -> Result<Vec<{StructName}>, Box<dyn std::error::Error>> {{
    let mut reader = Reader::from_path(source.as_ref())?;
    let mut res: Vec<{StructName}> = vec![];
    for iter in reader.records() {{
        match iter {{
            Ok(data) => {{
                let tmp: Vec<String> = data.into_iter().map(|s| s.trim().to_owned()).collect();
                res.push({StructName}::from(tmp));
            }}
            Err(e) => {{
                return Err(Box::new(e));
            }}
        }}
    }}

    Ok(res)
}}
"#,
        IMPORTS.join("\n"),
        &struct_tmpl,
        &from_impl,
        StructName = &struct_name
    );

    let mut fh = File::create(dist.as_ref())?;
    fh.write_all(module.as_bytes())?;

    Ok(module)
}

// HELPERS

/// Convert OS file name to applicable to struct name;
///
/// # Arguments
///
/// * `file_name` - OS like file name. (Probably file stem from fs lib)
///
/// # Return
/// String value of applicable rust struct name;
///
fn to_struct_name(file_name: &str) -> String {
    let trimmed = file_name.trim();

    let sanitized: String = trimmed
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();

    let mut struct_name = String::new();
    let mut capitalize_next = true;

    for c in sanitized.chars() {
        if capitalize_next {
            struct_name += &c.to_uppercase().to_string();
        } else {
            struct_name.push(c);
        }
        capitalize_next = c == '_';
    }

    struct_name.retain(|c| c != '_');

    struct_name
}

/// Convert csv headers strings to strings applicable to struct property names;
///
/// # Arguments
///
/// * `headers` - Common strings in vectors
///
/// # Return
/// Vector of strings which could be used as a struct property
///
fn parse_headers(headers: Vec<String>) -> Vec<String> {
    if headers.is_empty() {
        return Vec::default();
    }

    // Prove: pretty sure in correctness
    let incorrect_start = Regex::new(r"^[^a-z]+").unwrap();
    let unavailable_symbols = Regex::new(r"\W+").unwrap();
    let clear_endings = Regex::new(r"(^_+)|(_+$)").unwrap();

    headers
        .into_iter()
        .enumerate()
        .map(move |(i, s)| {
            let mut tmp = String::from(incorrect_start.replace_all(s.to_lowercase().trim(), ""));
            tmp = String::from(unavailable_symbols.replace_all(&tmp, "_"));
            tmp = String::from(clear_endings.replace_all(&tmp, ""));

            if tmp.is_empty() {
                tmp = format!("column_{}", i + 1);
            }

            tmp
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::{parse_col_type, parse_headers, DeserializationType};

    #[test]
    fn test_is_same_type() {
        let e1 = DeserializationType::EMPTY;
        let e2 = DeserializationType::EMPTY;

        assert!(e1.is_same_type(&e2));

        let n1 = DeserializationType::NUMBER(1.2345);
        let n2 = DeserializationType::NUMBER(3.345);

        assert!(n1.is_same_type(&n2));

        let s1 = DeserializationType::STRING("1.2345".to_owned());
        let n1 = DeserializationType::NUMBER(3.345);

        assert!(!s1.is_same_type(&n1));
    }

    #[test]
    fn test_header_parsing() {
        let raw = vec![
            "unique id",
            "kek lol chebureck!",
            "FALSE",
            "  ",
            "@@@@_Kekes_l___________",
        ]
        .into_iter()
        .map(|s| s.to_owned())
        .collect();
        let headers = parse_headers(raw);

        assert_eq!(
            headers,
            vec![
                "unique_id".to_owned(),
                "kek_lol_chebureck".to_owned(),
                "false".to_owned(),
                "column_4".to_owned(),
                "kekes_l".to_owned()
            ]
        )
    }

    #[test]
    fn test_value_parsing() {
        let raw = vec!["3", "3.3", "kek lol chebureck!", "FALSE", "  "];
        let types = raw
            .into_iter()
            .map(|s| parse_col_type(s).unwrap())
            .collect::<Vec<DeserializationType>>();

        assert_eq!(
            types,
            vec![
                DeserializationType::NUMBER(3_f64),
                DeserializationType::NUMBER(3.3),
                DeserializationType::STRING("kek lol chebureck!".to_owned()),
                DeserializationType::BOOLEAN(false),
                DeserializationType::EMPTY
            ]
        )
    }
}
