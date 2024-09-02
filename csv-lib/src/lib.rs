use std::{collections::HashMap, fs::File, path::Path};

use csv::{Reader, ReaderBuilder, Terminator};

pub mod constants;
pub mod deserialization;
pub mod error;

use deserialization::{
    compare_col_values, parse_col_value, DeserializationType, DeserializationValue,
};
use error::Result;

pub struct Toolkit {
    reader: Reader<File>,
    min: HashMap<String, DeserializationValue>,
    max: HashMap<String, DeserializationValue>,
}

impl Toolkit {
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

        let reader = ReaderBuilder::new()
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

        Ok(Self { reader, min, max })
    }

    fn preparing(
        reader: &mut Reader<File>,
        min: HashMap<String, DeserializationValue>,
        max: HashMap<String, DeserializationValue>,
    ) -> Result<()> {
        let headers: Vec<String> = reader
            .headers()?
            .into_iter()
            .map(|s| s.to_owned())
            .collect();

        for it in reader.records() {
            match it {
                Ok(res) => {
                    for (header, variable) in headers.iter().zip(res.into_iter()) {
                        let var = parse_col_value(variable)?.1;
                    }
                }
                Err(e) => todo!(),
            }
        }

        Ok(())
    }

    fn get_min(curr: DeserializationValue, pt: DeserializationValue) -> DeserializationValue {
        match compare_col_values(&curr, &pt) {
            Some(ord) => match ord {
                std::cmp::Ordering::Greater => pt,
                _ => curr,
            },
            None => curr,
        }
    }

    fn get_max(curr: DeserializationValue, pt: DeserializationValue) -> DeserializationValue {
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
