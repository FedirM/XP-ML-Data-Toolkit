use std::{collections::HashMap, fs::File, path::Path};

use csv::{Position, Reader, ReaderBuilder, Terminator};

pub mod constants;
pub mod deserialization;
pub mod error;


use deserialization::{
    parse_col_type, DeserializationType,
};
use error::{CustomError, Result};

type ColSpec = HashMap<usize, DeserializationType>; 

pub struct CsvToolkit {


    reader: Reader<File>,

    pub headers: Vec<String>,

    pub min: HashMap<String, DeserializationType>,
    pub max: HashMap<String, DeserializationType>,

    pub gaps: HashMap<usize, ColSpec>,
    pub outliers: HashMap<usize, ColSpec>,

    data_position: Position
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

        let mut data_position = Position::new();
        let mut headers = reader
            .headers()?
            .into_iter()
            .map(|s| s.to_owned())
            .collect();

        let pos = reader.position();

        data_position.set_byte(pos.byte());
        data_position.set_line(pos.line());
        data_position.set_record(pos.record());

        let mut min = HashMap::default();
        let mut max = HashMap::default();


        Self::preprocessing(&mut reader, &headers, &mut min, &mut max)?;

        Ok(Self { reader, headers, min, max, data_position, gaps: HashMap::default(), outliers: HashMap::default() })
    }

    pub fn set_min(&mut self, header: String, value: DeserializationType) -> Result<()> {
        if let Some(old_value) = self.min.get(&header) {
            if value.is_same_type(old_value) {
                self.min.insert(header, value);
            } else {
                return Err(Box::new(CustomError::new("Incompatible value tyeps!")));
            }
        } else {
            self.min.insert(header, value);
        }
        Ok(())
    }

    pub fn set_max(&mut self, header: String, value: DeserializationType) -> Result<()> {
        if let Some(old_value) = self.max.get(&header) {
            if value.is_same_type(old_value) {
                self.max.insert(header, value);
            } else {
                return Err(Box::new(CustomError::new("Incompatible value tyeps!")));
            }
        } else {
            self.max.insert(header, value);
        }
        Ok(())
    }

    pub fn postprocessing(&mut self) -> Result<()> {
        self.reset_reader()?;
        self.gaps.clear();
        self.outliers.clear();

        for (row_id, data_row) in self.reader.records().enumerate() {
            let data_row = data_row?;
            for ((col_id, col_name), value) in self.headers.iter().enumerate().zip(data_row.iter()) {
                let value = parse_col_type(value)?;

                if value == DeserializationType::EMPTY {
                    // Update gaps

                    if let Some(col_hash) = self.gaps.get_mut(&row_id) {
                        col_hash.insert(col_id, DeserializationType::EMPTY);
                    } else {
                        let mut col_hash: HashMap<usize, DeserializationType> = HashMap::new();
                        col_hash.insert(col_id, DeserializationType::EMPTY);
                        self.gaps.insert(row_id, col_hash);
                    }
                } else {

                    // Update outliers
                    if let Some(limit) = self.max.get(col_name) {
                        let limit = limit.clone();
                        if value > limit {
                            if let Some(col_hash) = self.outliers.get_mut(&row_id) {
                                col_hash.insert(col_id, limit);
                            } else {
                                let mut col_hash: HashMap<usize, DeserializationType> = HashMap::new();
                                col_hash.insert(col_id, limit);
                                self.outliers.insert(row_id, col_hash);
                            }
                        }
                    }

                    
                    if let Some(limit) = self.min.get(col_name) {
                        let limit = limit.clone();
                        if value < limit {
                            if let Some(col_hash) = self.outliers.get_mut(&row_id) {
                                col_hash.insert(col_id, limit);
                            } else {
                                let mut col_hash: HashMap<usize, DeserializationType> = HashMap::new();
                                col_hash.insert(col_id, limit);
                                self.outliers.insert(row_id, col_hash);
                            }
                        }
                    }
                }
            }
        }


        Ok(())
    }

    fn preprocessing(
        reader: &mut Reader<File>,
        headers: &Vec<String>,
        min: &mut HashMap<String, DeserializationType>,
        max: &mut HashMap<String, DeserializationType>,
    ) -> Result<()> {

        for it in reader.records() {
            let res = it?;
            for (header, variable) in headers.iter().zip(res.into_iter()) {
                let var = parse_col_type(variable)?;

                if header.contains("feat") {
                    println!("Value: {}", variable);
                }

                match var {
                    DeserializationType::NUMBER(_) => {
                        if let Some(curr) = min.get(header).take() {
                            min.insert(header.to_owned(), Self::min(curr.clone(), var.clone()));
                        } else {
                            min.insert(header.to_owned(), var.clone());
                        }
        
                        if let Some(curr) = max.get(header).take() {
                            max.insert(header.to_owned(), Self::max(curr.clone(), var.clone()));
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

    fn min(curr: DeserializationType, pt: DeserializationType) -> DeserializationType {
        match &curr.partial_cmp(&pt) {
            Some(ord) => match ord {
                std::cmp::Ordering::Greater => pt,
                _ => curr,
            },
            None => curr,
        }
    }

    fn max(curr: DeserializationType, pt: DeserializationType) -> DeserializationType {
        match &curr.partial_cmp(&pt) {
            Some(ord) => match ord {
                std::cmp::Ordering::Less => pt,
                _ => curr,
            },
            None => curr,
        }
    }

    fn reset_reader(&mut self) -> std::result::Result<(), csv::Error> {
        self.reader.seek(self.data_position.clone())
    }

    pub fn simple_iter(&mut self) -> Result<()> {
        self.reset_reader()?;

        println!("H: {:?}", self.reader.headers()?);

        for r in self.reader.records() {
            println!("R: {:?}", r?);
        }
        
        Ok(())
    }
}
