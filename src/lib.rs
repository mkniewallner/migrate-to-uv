mod cli;
mod converters;
mod detector;
mod logger;
mod schema;
mod toml;

use crate::cli::cli;

pub fn main() {
    cli();
}
