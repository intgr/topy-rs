use crate::tagsoup::{parse_and_compile, TypoRule};
use log::{debug, error, info};
use rayon::prelude::*;
use std::borrow::Cow;
use std::env::args_os;
use std::error::Error;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::Read;
use std::io::Write;

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

fn apply_file_inner(rules: &[TypoRule], path: &OsString) -> Result<u32, AnyErr> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    let mut total_count = 0u32;
    file.read_to_string(&mut contents)?;

    for rule in rules {
        let result = rule.regex.replace_all(&contents, &rule.replace);
        // Borrowed value was returned -- no changes were made, ignore.
        if let Cow::Borrowed(_) = result {
            continue;
        }
        if contents == result {
            debug!(
                "{}: '{}' had a no-op match",
                path.to_string_lossy(),
                rule.label
            );
            continue;
        }

        info!("{}: Applied '{}'", path.to_string_lossy(), rule.label);
        total_count += 1;
        contents = result.into();
    }

    // TODO: Overwriting files shouldn't be the default behavior
    if total_count > 0 {
        let mut file = OpenOptions::new().write(true).truncate(true).open(path)?;
        file.write_all(contents.as_bytes())?;
        file.flush()?;
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
