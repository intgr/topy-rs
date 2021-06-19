use crate::tagsoup::parse_and_compile;

mod logging;
mod tagsoup;

const BUNDLED_RULES: &str = include_str!("../data/retf.txt");

fn main() {
    logging::init_with_level(log::Level::Debug);
    parse_and_compile(BUNDLED_RULES);
}
