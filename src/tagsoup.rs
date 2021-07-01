use fancy_regex::Error::LookBehindNotConst;
use fancy_regex::Regex;
use log::{debug, info, trace};
use rayon::prelude::*;
use std::borrow::Cow;

#[derive(Debug)]
/// Temporary struct
struct RawRule {
    // word=
    label: String,
    // find=
    pattern: String,
    // replace=
    replace: String,
}

/// Result of parsing
pub struct TypoRule {
    pub label: String,
    pub regex: Regex,
    pub replace: String,
}

struct FileParseResult {
    rules: Vec<RawRule>,
    // stats
    disabled: u32,
    tag_errors: u32,
}

struct CompileResult {
    rules: Vec<TypoRule>,
    backref_errors: u32,
    regex_errors: u32,
}

/// Parse rules file and compile Regexes
pub fn parse_and_compile(text: &str) -> Vec<TypoRule> {
    let result = parse_rules_file(text);
    let compiled = compile_rules(result.rules);

    info!(
        "Loaded rules: {}, disabled: {}, errors: (backref: {}, regex: {}, tag: {})",
        compiled.rules.len(),
        result.disabled,
        compiled.backref_errors,
        compiled.regex_errors,
        result.tag_errors
    );
    compiled.rules
}

fn parse_rules_file(text: &str) -> FileParseResult {
    // <Typo word="bias" find="\b([bB])iais\b" replace="$1ias"/>
    let tag_re = Regex::new(r#"<Typo(\s+[a-z_-]+="[^"\n]*")+\s*/>"#).unwrap();
    let attr_re = Regex::new(r#"([a-z_-]+)="([^"\n]*)""#).unwrap();

    // Results
    let mut rules = Vec::<RawRule>::new();
    let mut tag_errors = 0u32;
    let mut disabled = 0u32;

    'outer: for tag_cap in tag_re.captures_iter(text) {
        let tag = tag_cap.unwrap().get(0).unwrap().as_str();
        let mut label: Option<&str> = None;
        let mut pattern: Option<&str> = None;
        let mut replace: Option<&str> = None;

        for attr_cap in attr_re.captures_iter(tag) {
            let attr = attr_cap.unwrap();
            let key = attr.get(1).unwrap().as_str();
            // TODO: Handle escaped XML entities
            let value = attr.get(2).unwrap().as_str();

            if key == "word" {
                label = Some(value);
            } else if key == "find" {
                pattern = Some(value);
            } else if key == "replace" {
                replace = Some(value);
            } else if key == "disabled" || key == "disable" {
                trace!("Rule disabled: {}", tag);
                disabled += 1;
                continue 'outer;
            }
        }

        if label.is_none() {
            debug!("Rule is missing 'word': {}", tag);
            tag_errors += 1;
            continue;
        }
        let label = label.unwrap().to_string();

        if pattern.is_none() {
            debug!("Rule is missing 'find': {}", tag);
            tag_errors += 1;
            continue;
        }
        let pattern = pattern.unwrap().to_string();

        if replace.is_none() {
            debug!("Rule is missing 'replace': {}", tag);
            tag_errors += 1;
            continue;
        }
        let replace = replace.unwrap().to_string();

        rules.push(RawRule {
            label,
            pattern,
            replace,
        })
    }

    FileParseResult {
        rules,
        disabled,
        tag_errors,
    }
}

lazy_static! {
    static ref SUBSTITUTION_RE: Regex = Regex::new(r#"\$([0-9]+)"#).unwrap();
}

/// Replace substitution group references like `$1` with `${1}`.
/// Because regex and fancy-regex replacement syntax differs from the one expected by Wikipedia
/// rulesets.
///
/// See: https://github.com/rust-lang/regex/issues/69
fn convert_replace_string(input: String) -> String {
    match SUBSTITUTION_RE.replace_all(input.as_str(), "$${$1}") {
        Cow::Borrowed(_) => input, // Unchanged -- return input without copying
        Cow::Owned(val) => val,
    }
}

fn compile_rules(rules: Vec<RawRule>) -> CompileResult {
    let (results, errors): (Vec<_>, Vec<_>) = rules
        .into_par_iter()
        .map(move |raw| -> Result<TypoRule, fancy_regex::Error> {
            Ok(TypoRule {
                label: raw.label,
                regex: Regex::new(raw.pattern.as_str())?,
                replace: convert_replace_string(raw.replace),
            })
        })
        .partition(Result::is_ok);

    let mut regex_errors = 0u32;
    let mut backref_errors = 0u32;

    for error in errors {
        match error {
            Err(LookBehindNotConst) => {
                // debug!("Lookbehind error: rule '{}' regex '{}'", label, pattern);
                backref_errors += 1;
            }
            _err => {
                // debug!("Regex error in '{}': {}", label, err);
                regex_errors += 1;
            }
        }
    }

    CompileResult {
        rules: results.into_iter().map(Result::unwrap).collect(),
        regex_errors,
        backref_errors,
    }
}
