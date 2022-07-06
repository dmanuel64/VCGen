use crate::git::vulnerability::VulnerableCode;
use indicatif::ProgressBar;
use polars::{
    datatypes::{DataType, Utf8Chunked},
    io::SerWriter,
    prelude::{CsvWriter, DataFrame, IntoSeries, JsonWriter, NamedFrom, Series},
};
use std::{fs::File, iter::FromIterator, path::Path};

pub fn create_dataset(
    vulnerabilities: Vec<VulnerableCode>,
    progress: Option<&ProgressBar>,
) -> DataFrame {
    let mut git_url_col = Series::new_empty("GitHub URL", &DataType::Utf8);
    let mut commit_hash_col = Series::new_empty("Commit Hash", &DataType::Utf8);
    let mut repo_file_col = Series::new_empty("File", &DataType::Utf8);
    let mut code_col = Series::new_empty("Code", &DataType::Utf8);
    let mut flawfinder_vulnerabilities_col =
        Series::new_empty("Flawfinder Vulnerabilities", &DataType::Utf8);
    let mut flawfinder_cwes_col = Series::new_empty("Flawfinder CWEs", &DataType::Utf8);
    Vec::from_iter(vulnerabilities.iter().map(|vulnerability| {
        git_url_col
            .append(&Series::new(
                git_url_col.name(),
                &[vulnerability.repo_url()],
            ))
            .expect(&format!(
                "Could not add {} to dataset GitHub URLs",
                vulnerability.repo_url()
            ));
        commit_hash_col.append(&Series::new(
            commit_hash_col.name(),
            &[vulnerability.commit_hash()],
        ));
        code_col.append(&Series::new("", &[vulnerability.code()]));
        repo_file_col
            .append(&Series::new(
                repo_file_col.name(),
                &[vulnerability.file_path().to_str().unwrap()],
            ))
            .expect(&format!(
                "Could not add {} to dataset Files",
                vulnerability.file_path().to_str().unwrap()
            ));
        flawfinder_vulnerabilities_col.append(&Series::new(
            flawfinder_vulnerabilities_col.name(),
            &[vulnerability
                .flawfinder_results()
                .and_then(|v| Some(v.join(" ")))],
        ));
        flawfinder_cwes_col.append(&Series::new(
            flawfinder_vulnerabilities_col.name(),
            &[vulnerability
                .flawfinder_cwes()
                .and_then(|v| Some(v.join(" ")))],
        ));
        if let Some(pb) = progress {
            pb.inc(1);
        }
    }));
    let df = DataFrame::new(vec![
        git_url_col,
        commit_hash_col,
        repo_file_col,
        code_col,
        flawfinder_vulnerabilities_col,
        flawfinder_cwes_col,
    ])
    .unwrap();
    df
}

pub fn save_dataset(df: &mut DataFrame, dataset_path: &Path) {
    let f = File::create(dataset_path).unwrap();
    JsonWriter::new(f).finish(df);
}
