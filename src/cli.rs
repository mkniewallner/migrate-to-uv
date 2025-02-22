use crate::converters::{ConverterOptions, DependencyGroupsStrategy};
use crate::detector::{PackageManager, get_converter};
use crate::logger;
use clap::Parser;
use clap::builder::Styles;
use clap::builder::styling::{AnsiColor, Effects};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::error;
use std::path::PathBuf;
use std::process;

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Cyan.on_default())
    .error(AnsiColor::Red.on_default().effects(Effects::BOLD))
    .valid(AnsiColor::Cyan.on_default().effects(Effects::BOLD))
    .invalid(AnsiColor::Yellow.on_default().effects(Effects::BOLD));

#[derive(Parser)]
#[command(version)]
#[command(about = "Migrate a project to uv from another package manager.", long_about = None)]
#[command(styles = STYLES)]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[arg(default_value = ".", help = "Path to the project to migrate")]
    path: PathBuf,
    #[arg(
        long,
        help = "Shows what changes would be applied, without modifying files"
    )]
    dry_run: bool,
    #[arg(
        long,
        help = "Do not lock dependencies with uv at the end of the migration"
    )]
    skip_lock: bool,
    #[arg(
        long,
        help = "Skip checks for whether or not the project is already using uv"
    )]
    skip_uv_checks: bool,
    #[arg(
        long,
        help = "Ignore current locked versions of dependencies when generating `uv.lock`"
    )]
    ignore_locked_versions: bool,
    #[arg(
        long,
        help = "Replace existing data in `[project]` section of `pyproject.toml` instead of keeping existing fields"
    )]
    replace_project_section: bool,
    #[arg(
        long,
        help = "Enforce a specific package manager instead of auto-detecting it"
    )]
    package_manager: Option<PackageManager>,
    #[arg(
        long,
        default_value = "set-default-groups",
        help = "Strategy to use when migrating dependency groups"
    )]
    dependency_groups_strategy: DependencyGroupsStrategy,
    #[arg(long, help = "Keep data from current package manager")]
    keep_current_data: bool,
    #[arg(long, default_values = vec!["requirements.txt"], help = "Requirements file to migrate")]
    requirements_file: Vec<String>,
    #[arg(long, default_values = vec!["requirements-dev.txt"], help = "Development requirements file to migrate")]
    dev_requirements_file: Vec<String>,
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

pub fn cli() {
    let cli = Cli::parse();

    logger::configure(cli.verbose);

    let converter_options = ConverterOptions {
        project_path: PathBuf::from(&cli.path),
        dry_run: cli.dry_run,
        skip_lock: cli.skip_lock,
        skip_uv_checks: cli.skip_uv_checks,
        ignore_locked_versions: cli.ignore_locked_versions,
        replace_project_section: cli.replace_project_section,
        keep_old_metadata: cli.keep_current_data,
        dependency_groups_strategy: cli.dependency_groups_strategy,
    };

    match get_converter(
        &converter_options,
        cli.requirements_file,
        cli.dev_requirements_file,
        cli.package_manager,
    ) {
        Ok(converter) => {
            converter.convert_to_uv();
        }
        Err(error) => {
            error!("{}", error);
            process::exit(1);
        }
    }

    process::exit(0)
}
