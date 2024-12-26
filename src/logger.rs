use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::Level;
use owo_colors::OwoColorize;
use std::io::Write;

pub fn configure(verbosity: Verbosity<InfoLevel>) {
    env_logger::Builder::new()
        .filter_level(verbosity.log_level_filter())
        .format(|buf, record| match record.level() {
            Level::Error => writeln!(buf, "{}: {}", "error".red().bold(), record.args()),
            Level::Warn => writeln!(buf, "{}: {}", "warning".yellow().bold(), record.args()),
            Level::Debug => writeln!(buf, "{}: {}", "debug".blue().bold(), record.args()),
            _ => writeln!(buf, "{}", record.args()),
        })
        .init();
}
