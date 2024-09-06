use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use constants::generate_temporary_file_name;
use csv::{Position, Reader, ReaderBuilder, Terminator};

pub mod constants;
pub mod deserialization;
pub mod error;
pub mod user_input;

use deserialization::{parse_col_type, DeserializationType};
use error::{CustomError, Result};

type ColSpec = HashMap<usize, DeserializationType>;

pub struct CsvToolkit {
    reader: Reader<File>,

    pub headers: Vec<String>,
    pub types: Vec<DeserializationType>,

    pub min: HashMap<String, DeserializationType>,
    pub max: HashMap<String, DeserializationType>,

    pub gaps: HashMap<usize, ColSpec>,
    pub outliers: HashMap<usize, ColSpec>,

    data_position: Position,
    delimiter: u8,
    comment: Option<u8>,
    double_quotes: bool,
    escape: Option<u8>,
    terminator: Terminator,

    tmp_file: PathBuf,
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
        let headers: Vec<String> = reader
            .headers()?
            .into_iter()
            .map(|s| s.to_owned())
            .collect();

        let pos = reader.position();

        data_position.set_byte(pos.byte());
        data_position.set_line(pos.line());
        data_position.set_record(pos.record());

        let row_len = headers.len();

        let mut toolkit = Self {
            reader,
            comment,
            escape,
            double_quotes,
            terminator,
            delimiter,
            headers,
            types: Vec::with_capacity(row_len),
            min: HashMap::with_capacity(row_len),
            max: HashMap::with_capacity(row_len),
            data_position,
            gaps: HashMap::default(),
            outliers: HashMap::default(),
            tmp_file: PathBuf::from(generate_temporary_file_name()),
        };

        toolkit.preprocessing()?;

        Ok(toolkit)
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
            for ((col_id, col_name), value) in self.headers.iter().enumerate().zip(data_row.iter())
            {
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
                    if !value.is_ordered() {
                        continue;
                    }

                    // self.update_outliers(row_id, col_id, col_name, value);

                    if let Some(limit) = self.max.get(col_name) {
                        let limit = limit.clone();
                        if value > limit {
                            if let Some(col_hash) = self.outliers.get_mut(&row_id) {
                                col_hash.insert(col_id, limit);
                            } else {
                                let mut col_hash: HashMap<usize, DeserializationType> =
                                    HashMap::new();
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
                                let mut col_hash: HashMap<usize, DeserializationType> =
                                    HashMap::new();
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

    pub fn normalizing(&mut self, column_list: Vec<String>) -> Result<()> {
        self.reset_reader()?;

        // Check if all columns is NUMBER
        for h in column_list.iter() {
            if let Some(min_value) = self.min.get(h) {
                if !min_value.is_ordered() {
                    return Err(Box::new(CustomError::new(&format!(
                        "Column '{h}' could not be normalized!"
                    ))));
                }
            }
        }

        let fh = File::create(self.tmp_file.as_path())?;
        let delimiter = String::from(core::str::from_utf8(&[self.delimiter]).unwrap());

        println!("DELIMITER: {}", delimiter);

        self.reset_reader()?;

        let mut buf_writer = BufWriter::new(fh);
        let mut row: Vec<String> = Vec::with_capacity(self.headers.len());

        // Write CSV headers
        writeln!(&mut buf_writer, "{}", &self.headers.join(&delimiter))?;
        println!("HEADERS: {}", &self.headers.join(&delimiter));

        for (_row_id, it) in self.reader.records().enumerate() {
            let string_record = it?;
            row.clear();

            for ((_col_id, header), string_value) in self
                .headers
                .iter()
                .enumerate()
                .zip(string_record.into_iter())
            {
                if column_list.contains(header) {
                    let value = parse_col_type(string_value)?;
                    match value {
                        DeserializationType::NUMBER(_) => {
                            match (self.min.get(header), self.max.get(header)) {
                                (Some(min), Some(max)) => {
                                    let min = min.clone();
                                    let max = max.clone();

                                    let n_val =
                                        (value.clone() - min.clone()) / (max.clone() - min.clone());
                                    row.push(n_val.to_string());

                                    println!("N-val: {}", n_val);
                                }
                                _ => row.push(string_value.to_string()),
                            }
                        }
                        _ => row.push(string_value.to_string()),
                    }
                } else {
                    row.push(string_value.to_string());
                }
            }

            // Write data row (with normalized data)
            writeln!(&mut buf_writer, "{}", row.join(&delimiter))?;
            println!("DAT: {}", row.join(&delimiter));
        }

        buf_writer.flush()?;

        self.switch_reader_to_tmp_file()?;
        self.preprocessing()?;

        Ok(())
    }

    fn preprocessing(&mut self) -> Result<()> {
        self.min.clear();
        self.max.clear();
        self.types.clear();
        self.gaps.clear();

        for (row_id, it) in self.reader.records().enumerate() {
            let res = it?;
            for ((col_id, header), variable) in self.headers.iter().enumerate().zip(res.into_iter())
            {
                let var = parse_col_type(variable)?;

                Self::check_or_insert_column_type(&mut self.types, col_id, &header, &var)?;

                match var {
                    DeserializationType::NUMBER(_) => {
                        if let Some(curr) = self.min.get(header).take() {
                            self.min
                                .insert(header.to_owned(), min!(curr.clone(), var.clone()));
                        } else {
                            self.min.insert(header.to_owned(), var.clone());
                        }

                        if let Some(curr) = self.max.get(header).take() {
                            self.max
                                .insert(header.to_owned(), max!(curr.clone(), var.clone()));
                        } else {
                            self.max.insert(header.to_owned(), var.clone());
                        }
                    }
                    DeserializationType::EMPTY => {
                        if let Some(g_row) = self.gaps.get_mut(&row_id) {
                            g_row.insert(col_id, var.clone());
                        } else {
                            let mut tmp = HashMap::new();
                            tmp.insert(col_id, var.clone());
                            self.gaps.insert(row_id, tmp);
                        }
                    }
                    _ => continue,
                }
            }
        }

        Ok(())
    }

    /// Reset seek position for inner `reader` (csv::Reader) instance to be able to read src file one more.
    ///
    fn reset_reader(&mut self) -> std::result::Result<(), csv::Error> {
        self.reader.seek(self.data_position.clone())
    }

    /// Switch the `reader` (csv::Reader) obj to the temporary file.
    ///
    /// This method should call after all types of source data mutation!
    ///
    fn switch_reader_to_tmp_file(&mut self) -> Result<()> {
        self.reader = ReaderBuilder::new()
            .trim(csv::Trim::All)
            .terminator(self.terminator.clone())
            .flexible(true)
            .escape(self.escape.clone())
            .double_quote(self.double_quotes)
            .comment(self.comment.clone())
            .delimiter(self.delimiter)
            .from_path(self.tmp_file.as_path())?;

        Ok(())
    }

    /// Checks the type of a value in a column and ensures that all values in the column are of the same type. Add one if absent.
    ///
    /// This method compares the type of a new value with the existing type for the specified column. If the types
    /// do not match, it returns an error. If this is the first value for the column, it records the type of the value.
    ///
    /// # Arguments
    ///
    /// * `types` - A mutable reference to the current instance of `types` vector.
    /// * `column_id` - The index of the column to check.
    /// * `column_name` - The name of the column, used for error reporting.
    /// * `column_value` - A reference to the value whose type will be checked.
    ///
    /// # Errors
    ///
    /// Returns an error if the type of `column_value` does not match the previously recorded type for the column.
    /// The error message includes the column name and specifies that the types are inconsistent.
    ///

    fn check_or_insert_column_type(
        types: &mut Vec<DeserializationType>,
        column_id: usize,
        column_name: &str,
        column_value: &DeserializationType,
    ) -> Result<()> {
        if let Some(curr_type) = types.get(column_id) {
            if !curr_type.is_same_type(column_value) {
                return Err(Box::new(CustomError::new(&format!(
                    "Seems like in column '{}' not all values are the same type!",
                    column_name
                ))));
            }
        } else {
            types.insert(column_id, column_value.clone());
        }

        Ok(())
    }

    // fn update_outliers(
    //     &mut self,
    //     row_id: usize,
    //     column_id: usize,
    //     column_name: &str,
    //     current_value: DeserializationType,
    // ) {
    //     if let Some(limit) = self.max.get(column_name) {
    //         let limit = limit.clone();
    //         if current_value > limit {
    //             if let Some(col_hash) = self.outliers.get_mut(&row_id) {
    //                 col_hash.insert(column_id, limit);
    //             } else {
    //                 let mut col_hash: HashMap<usize, DeserializationType> = HashMap::new();
    //                 col_hash.insert(column_id, limit);
    //                 self.outliers.insert(row_id, col_hash);
    //             }
    //         }
    //     }

    //     if let Some(limit) = self.min.get(column_name) {
    //         let limit = limit.clone();
    //         if current_value < limit {
    //             if let Some(col_hash) = self.outliers.get_mut(&row_id) {
    //                 col_hash.insert(column_id, limit);
    //             } else {
    //                 let mut col_hash: HashMap<usize, DeserializationType> = HashMap::new();
    //                 col_hash.insert(column_id, limit);
    //                 self.outliers.insert(row_id, col_hash);
    //             }
    //         }
    //     }
    // }
}

#[cfg(test)]
pub mod test {

    use super::*;
    use std::fs;
    use std::path::Path;

    fn init() -> Result<CsvToolkit> {
        return CsvToolkit::new(Path::new("./tests/test.csv"), b',', None, false, None, None);
    }

    #[test]
    pub fn initialize_test() {
        match init() {
            Ok(_) => assert!(true),
            Err(e) => assert!(false, "Could not initiate CsvToolkit!\n{e}"),
        }
    }

    #[test]
    pub fn test_min_max() {
        match init() {
            Ok(toolkit) => {
                let test_key = String::from("Exercise Hours Per Week");

                assert_eq!(
                    toolkit.min.get(&test_key),
                    Some(&DeserializationType::NUMBER(0.1945150606299495))
                );

                assert_eq!(
                    toolkit.max.get(&test_key),
                    Some(&DeserializationType::NUMBER(19.633268156072297))
                )
            }
            Err(e) => assert!(false, "Could not initiate CsvToolkit!\n{e}"),
        }
    }

    #[test]
    pub fn test_normalization() {
        match init() {
            Ok(mut toolkit) => {
                let test_key = String::from("Exercise Hours Per Week");
                let tmp_file = toolkit.tmp_file.clone();

                match toolkit.normalizing(vec![test_key.clone()]) {
                    Ok(_) => {
                        fs::remove_file(tmp_file.as_path()).unwrap();
                        assert_eq!(
                            toolkit.min.get(&test_key),
                            Some(&DeserializationType::NUMBER(0_f64))
                        );

                        assert_eq!(
                            toolkit.max.get(&test_key),
                            Some(&DeserializationType::NUMBER(1_f64))
                        )
                    }
                    Err(e) => assert!(false, "{e}"),
                }
            }
            Err(e) => assert!(false, "Could not initiate CsvToolkit!\n{e}"),
        }
    }
}
