use insta_cmd::get_cargo_bin;
use serde::Deserialize;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

macro_rules! apply_filters {
    {} => {
        let mut settings = insta::Settings::clone_current();
        settings.add_filter(r"Using .+", "Using [PYTHON_INTERPRETER]");
        settings.add_filter(r"Defaulting to `\S+`", "Defaulting to `[PYTHON_VERSION]`");
        settings.add_filter(r"Resolved \d+ package in \S+", "Resolved [PACKAGES] package in [TIME]");
        settings.add_filter(r"Resolved \d+ packages in \S+", "Resolved [PACKAGES] packages in [TIME]");
        settings.add_filter(r"Updated https://github.com/encode/uvicorn (\S+)", "Updated https://github.com/encode/uvicorn ([SHA1])");
        settings.add_filter(r"uv_build>=[\d\.]+,<[\d\.]+", "uv_build>=[LOWER_BOUND],<[UPPER_BOUND]");
        let _bound = settings.bind_to_scope();
    }
}

pub(crate) use apply_filters;

#[allow(dead_code)]
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct UvLock {
    pub package: Option<Vec<LockedPackage>>,
}

#[allow(dead_code)]
#[derive(Deserialize, Eq, PartialEq, Debug)]
pub struct LockedPackage {
    pub name: String,
    pub version: String,
}

#[allow(dead_code)]
pub enum PackageBuilder {
    Hatch,
    Poetry,
    Uv,
}

impl PackageBuilder {
    #[allow(dead_code)]
    pub fn get_command(&self) -> Command {
        match self {
            PackageBuilder::Hatch => {
                let mut command = Command::new("uvx");
                command.arg("hatch").arg("build");
                command
            }
            PackageBuilder::Poetry => {
                let mut command = Command::new("uvx");
                command.arg("poetry").arg("build");
                command
            }
            PackageBuilder::Uv => {
                let mut command = Command::new("uv");
                command.arg("build");
                command
            }
        }
    }
}

#[allow(dead_code)]
pub fn build_packages(builder: &PackageBuilder, project_path: &Path) -> ExitStatus {
    builder
        .get_command()
        .current_dir(project_path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap()
}

pub fn cli() -> Command {
    Command::new(get_cargo_bin("migrate-to-uv"))
}
