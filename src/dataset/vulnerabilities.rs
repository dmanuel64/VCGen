use crate::{git::vulnerability::AnalyzedFile, utils::set_optional_message};
use indicatif::ProgressBar;
use polars::{
    datatypes::DataType,
    io::SerWriter,
    prelude::{DataFrame, JsonWriter, NamedFrom, Series},
};
use std::{fs::File, iter::FromIterator, path::Path};

/// Adds a string to a [`Series`].
fn add_string_to_series(series: &mut Series, e: Option<&str>) {
    series
        .append(&mut Series::new("", &[e]))
        .expect(&format!("Could not add {:?} to dataset", e));
}

/// Adds a vector to a [`Series`] as a string.
fn add_vec_to_series(series: &mut Series, v: Option<&Vec<String>>) {
    series
        .append(&mut Series::new("", &[v.and_then(|e| Some(e.join(" ")))]))
        .expect(&format!("Could not add {:?} to dataset", v));
}

/// Creates a [`DataFrame`] of vulnerable/benign code.
pub fn create_dataset(
    vulnerabilities: Vec<AnalyzedFile>,
    progress: Option<&ProgressBar>,
) -> DataFrame {
    // create DataFrame columns
    let mut git_url_col = Series::new_empty("GitHub URL", &DataType::Utf8);
    let mut commit_hash_col = Series::new_empty("Commit Hash", &DataType::Utf8);
    let mut repo_file_col = Series::new_empty("File", &DataType::Utf8);
    let mut code_col = Series::new_empty("Code", &DataType::Utf8);
    let mut flawfinder_vulnerabilities_col =
        Series::new_empty("Flawfinder Vulnerabilities", &DataType::Utf8);
    let mut flawfinder_cwes_col = Series::new_empty("Flawfinder CWEs", &DataType::Utf8);
    set_optional_message(progress, "Building vulnerable code dataset");
    // Iterate through vulnerabilities and add them to their respective columns
    Vec::from_iter(vulnerabilities.iter().map(|vulnerability| {
        add_string_to_series(&mut git_url_col, Some(vulnerability.repo_url()));
        add_string_to_series(&mut commit_hash_col, Some(vulnerability.commit_hash()));
        add_string_to_series(
            &mut repo_file_col,
            Some(
                vulnerability
                    .repo_file_path()
                    .to_str()
                    .expect("Could not get vulnerable repo file path."),
            ),
        );
        add_string_to_series(&mut code_col, Some(vulnerability.code()));
        add_vec_to_series(
            &mut flawfinder_vulnerabilities_col,
            vulnerability.flawfinder_results(),
        );
        add_vec_to_series(&mut flawfinder_cwes_col, vulnerability.flawfinder_cwes());
        if let Some(pb) = progress {
            pb.inc(1);
        }
    }));
    // create DataFrame
    let df = DataFrame::new(vec![
        git_url_col,
        commit_hash_col,
        repo_file_col,
        code_col,
        flawfinder_vulnerabilities_col,
        flawfinder_cwes_col,
    ])
    .expect("Could not create DataFrame.");
    df
}

/// Saves a [`DataFrame`] to a specified [`Path`].
pub fn save_dataset(df: &mut DataFrame, dataset_path: &Path) {
    let f = File::create(dataset_path).unwrap();
    JsonWriter::new(f)
        .finish(df)
        .expect(&format!("Could not save dataset to {:?}", dataset_path));
}
