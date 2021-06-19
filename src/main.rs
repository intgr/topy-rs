use crate::tagsoup::{parse_and_compile, TypoRule};
use fancy_regex::Regex;
use log::{debug, error, info};
use rayon::prelude::*;
use std::env::args_os;
use std::error::Error;
use std::ffi::OsString;
use std::fs::File;
use std::io::Read;

mod logging;
mod tagsoup;

pub type AnyErr = Box<dyn Error>;

const BUNDLED_RULES: &str = include_str!("../data/retf.txt");

fn main() {
    logging::init_with_level(log::Level::Debug);
    let rules = parse_and_compile(BUNDLED_RULES);

    let args: Vec<OsString> = args_os().skip(1).collect();
    apply_files(&rules, args);
}

fn apply_files(rules: &[TypoRule], paths: Vec<OsString>) -> u32 {
    paths
        .into_par_iter()
        .map(|path| apply_file(rules, path))
        .sum()
}

fn regex_replace(input: String, regex: &Regex, replace: &str) -> (String, u32) {
    let content = input.clone();
    let mut count = 0u32;

    for mat in regex.find_iter(&input.as_str()) {
        let mat = mat.unwrap();
        // FIXME implement capture groups in replace
        // FIXME need to rejigger the ranges of successive matches
        // content.replace_range(mat.range(), replace);
        debug!("{} -> {}", mat.as_str(), replace);
        count += 1;
    }
    (content, count)
}

fn apply_file_inner(rules: &[TypoRule], path: &OsString) -> Result<u32, AnyErr> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    let mut total_count = 0u32;
    file.read_to_string(&mut contents)?;

    for rule in rules {
        let result = regex_replace(contents, &rule.regex, &rule.replace);
        contents = result.0;
        let count = result.1;
        total_count += count;
        if count > 0 {
            info!(
                "{}: Applied {} x '{}'",
                path.to_string_lossy(),
                count,
                rule.label
            );
        }
    }
    Ok(total_count)
}

fn apply_file(rules: &[TypoRule], path: OsString) -> u32 {
    match apply_file_inner(rules, &path) {
        Err(err) => {
            error!("Error applying file '{}': {}", path.to_string_lossy(), err);
            0
        }
        Ok(count) => count,
    }
}
