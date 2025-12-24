mod cli;
mod converters;
mod detector;
mod errors;
mod logger;
mod schema;
mod toml;
mod uv;

use crate::cli::cli;

pub fn main() {
    cli();
}
