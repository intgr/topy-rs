use fancy_regex::Regex;
use log::{debug, info, trace};

#[derive(Debug)]
pub struct TypoRule<'a> {
    // word=
    label: &'a str,
    // find=
    pattern: &'a str,
    // replace=
    replace: &'a str,
}

pub fn parse_rules(text: &str) -> Vec<TypoRule> {
    let mut rules = Vec::<TypoRule>::new();
    let mut errors = 0;
    let mut disabled = 0;
    // <Typo word="bias" find="\b([bB])iais\b" replace="$1ias"/>
    let tag_re = Regex::new(r#"<Typo(\s+[a-z_-]+="[^"\n]*")+\s*/>"#).unwrap();
    let attr_re = Regex::new(r#"([a-z_-]+)="([^"\n]*)""#).unwrap();

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
            errors += 1;
            continue;
        }
        let label = label.unwrap();
        if pattern.is_none() {
            debug!("Rule is missing 'find': {}", tag);
            errors += 1;
            continue;
        }
        let pattern = pattern.unwrap();
        if replace.is_none() {
            debug!("Rule is missing 'replace': {}", tag);
            errors += 1;
            continue;
        }
        let replace = replace.unwrap();

        // TODO: Do something with the Regex
        if let Err(err) = Regex::new(pattern) {
            errors += 1;
            debug!("Error parsing '{}' rule: {}", label, err);
        }

        let rule = TypoRule {
            label,
            pattern,
            replace,
        };
        rules.push(rule);
    }

    info!(
        "Finished (rules: {}, errors: {}, disabled: {})",
        rules.len(),
        errors,
        disabled
    );
    rules
}
