use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use csv::Reader;
use regex::Regex;

use crate::constants::{IMPORTS, STRUCT_DERIVE};
use crate::error::CustomError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeserializationType {
    INTEGER,
    FLOATING,
    BOOLEAN,
    STRING,
}

impl std::fmt::Display for DeserializationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let col_type = match self {
            DeserializationType::INTEGER => "i32".to_owned(),
            DeserializationType::FLOATING => "f64".to_owned(),
            DeserializationType::BOOLEAN => "bool".to_owned(),
            DeserializationType::STRING => "String".to_owned(),
        };

        write!(f, "{col_type}")
    }
}

pub(crate) fn detect_col_type(value: &str) -> DeserializationType {
    let int_re = Regex::new(r"^\d+$").unwrap();
    let float_re = Regex::new(r"^(\d+)?\.\d+$").unwrap();

    match value {
        // TODO: For INT & FLOAT provide more viable check including num separation by [\s,_]
        _ if int_re.is_match(value) => DeserializationType::INTEGER,
        _ if float_re.is_match(value) => DeserializationType::FLOATING,
        "FALSE" | "False" | "false" | "TRUE" | "True" | "true" => DeserializationType::BOOLEAN,
        _ => DeserializationType::STRING,
    }
}

pub fn generate_struct<F: AsRef<Path>>(
    source: F,
    dist: F,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_path(source.as_ref())?;
    let mut headers: Vec<String> = reader
        .headers()?
        .clone()
        .into_iter()
        .map(|s| s.to_owned())
        .collect();
    headers = parse_headers(headers);

    let data_row: Vec<String> = if let Some(data) = reader.records().next() {
        data?.into_iter().map(|s| s.trim().to_owned()).collect()
    } else {
        return Err(Box::new(CustomError::new(
            "No data were found in csv file!",
        )));
    };

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
        let value_type = detect_col_type(&data_row[id]);
        struct_tmpl += &format!("\n\t{h_item}: {value_type},",);
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
pub fn init(source: impl AsRef<Path>) -> Result<Vec<{}>, Box<dyn std::error::Error>> {{
    let mut reader = Reader::from_path(source.as_ref())?;
    let mut res: Vec<{}> = vec![];
    for iter in reader.records() {{
        match iter {{
            Ok(data) => {{
                let tmp: Vec<String> = data.into_iter().map(|s| s.trim().to_owned()).collect();
                res.push({}::from(tmp));
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
        &struct_name,
        &struct_name,
        &struct_name
    );

    let mut fh = File::create(dist.as_ref())?;
    fh.write_all(module.as_bytes())?;

    Ok(module)
}

// HELPERS

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

fn parse_headers(headers: Vec<String>) -> Vec<String> {
    if headers.is_empty() {
        return Vec::default();
    }

    // Prove: pretty sure in correctness
    let incorrect_start = Regex::new(r"^[^a-z]+").unwrap();
    let unavailable_symbols = Regex::new(r"\W+").unwrap();

    headers
        .into_iter()
        .map(move |s| {
            let clean_start =
                String::from(incorrect_start.replace_all(s.to_lowercase().trim(), ""));
            String::from(unavailable_symbols.replace_all(&clean_start, "_"))
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::{detect_col_type, DeserializationType};

    #[test]
    fn test_type() {
        let raw = vec!["3", "3.3", "kek lol chebureck!", "FALSE"];
        let types = raw
            .into_iter()
            .map(|s| detect_col_type(s))
            .collect::<Vec<DeserializationType>>();

        assert_eq!(
            types,
            vec![
                DeserializationType::INTEGER,
                DeserializationType::FLOATING,
                DeserializationType::STRING,
                DeserializationType::BOOLEAN
            ]
        )
    }
}
