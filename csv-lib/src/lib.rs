use std::{collections::HashMap, fs::File, path::Path};

use csv::{Reader, ReaderBuilder, Terminator};

pub mod constants;
pub mod deserialization;
pub mod error;

use deserialization::{
    compare_col_values, parse_col_type, DeserializationType,
};
use error::Result;

pub struct CsvToolkit {
    reader: Reader<File>,
    pub min: HashMap<String, DeserializationType>,
    pub max: HashMap<String, DeserializationType>,
}

impl CsvToolkit {
    pub fn new(
        src: impl AsRef<Path>,
        delimiter: u8,
        comment: Option<u8>,
        double_quotes: bool,
        escape: Option<u8>,
        terminator: Option<u8>,
    ) -> Result<Self> {
        let terminator = match terminator {
            Some(s) => Terminator::Any(s),
            None => Terminator::CRLF,
        };

        let mut reader = ReaderBuilder::new()
            .trim(csv::Trim::All)
            .terminator(terminator)
            .flexible(true)
            .escape(escape)
            .double_quote(double_quotes)
            .comment(comment)
            .delimiter(delimiter)
            .from_path(src)?;

        let mut min = HashMap::default();
        let mut max = HashMap::default();

        Self::preprocessing(&mut reader, &mut min, &mut max)?;

        Ok(Self { reader, min, max })
    }

    fn preprocessing(
        reader: &mut Reader<File>,
        min: &mut HashMap<String, DeserializationType>,
        max: &mut HashMap<String, DeserializationType>,
    ) -> Result<()> {
        let headers: Vec<String> = reader
            .headers()?
            .into_iter()
            .map(|s| s.to_owned())
            .collect();

        for it in reader.records() {
            let res = it?;
            for (header, variable) in headers.iter().zip(res.into_iter()) {
                let var = parse_col_type(variable)?;

                if header.contains("feat") {
                    println!("Value: {}", variable);
                }

                match var {
                    DeserializationType::FLOATING(_) | DeserializationType::INTEGER(_) => {
                        if let Some(curr) = min.get(header).take() {
                            min.insert(header.to_owned(), Self::get_min(curr.clone(), var.clone()));
                        } else {
                            min.insert(header.to_owned(), var.clone());
                        }
        
                        if let Some(curr) = max.get(header).take() {
                            max.insert(header.to_owned(), Self::get_max(curr.clone(), var.clone()));
                        } else {
                            max.insert(header.to_owned(), var.clone());
                        }
                    },
                    _ => continue
                }
            }
        }

        Ok(())
    }

    fn get_min(curr: DeserializationType, pt: DeserializationType) -> DeserializationType {
        match compare_col_values(&curr, &pt) {
            Some(ord) => match ord {
                std::cmp::Ordering::Greater => pt,
                _ => curr,
            },
            None => curr,
        }
    }

    fn get_max(curr: DeserializationType, pt: DeserializationType) -> DeserializationType {
        match compare_col_values(&curr, &pt) {
            Some(ord) => match ord {
                std::cmp::Ordering::Less => pt,
                _ => curr,
            },
            None => curr,
        }
    }

    fn check_limits(&mut self) {}
}
