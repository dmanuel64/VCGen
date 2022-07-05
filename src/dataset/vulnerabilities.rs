use std::{fs::File, path::Path};

use crate::git::vulnerability::VulnerableCode;
use polars::{
    datatypes::DataType,
    prelude::{DataFrame, Series, CsvWriter}, io::SerWriter
};

pub fn save_dataset(vulnerabilities: Vec<VulnerableCode>, dataset_path: &Path) -> DataFrame {
    let mut df = DataFrame::new(vec![Series::new_empty("a", &DataType::Utf8)]).unwrap();
    let f = File::create(dataset_path).unwrap();
    CsvWriter::new(f).finish(&mut df);
    df
}
