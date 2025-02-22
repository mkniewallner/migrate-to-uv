use insta_cmd::get_cargo_bin;
use serde::Deserialize;
use std::process::Command;

macro_rules! apply_lock_filters {
    {} => {
        let mut settings = insta::Settings::clone_current();
        settings.add_filter(r"Using .+", "Using [PYTHON_INTERPRETER]");
        settings.add_filter(r"Defaulting to `\S+`", "Defaulting to `[PYTHON_VERSION]`");
        settings.add_filter(r"Resolved \d+ packages in \S+", "Resolved [PACKAGES] packages in [TIME]");
        settings.add_filter(r"Updated https://github.com/encode/uvicorn (\S+)", "Updated https://github.com/encode/uvicorn ([SHA1])");
        let _bound = settings.bind_to_scope();
    }
}

pub(crate) use apply_lock_filters;

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct UvLock {
    pub package: Option<Vec<LockedPackage>>,
}

#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
}

pub fn cli() -> Command {
    Command::new(get_cargo_bin("migrate-to-uv"))
}
