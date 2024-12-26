use crate::converters::{get_converter, DependencyGroupsStrategy};
use crate::detector::{Detector, PackageManager};
use crate::logger;
use clap::builder::styling::{AnsiColor, Effects};
use clap::builder::Styles;
use clap::Parser;
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
    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

pub fn cli() {
    let cli = Cli::parse();

    logger::configure(cli.verbose);

    let detector = Detector {
        project_path: &cli.path,
        enforced_package_manager: cli.package_manager,
    };

    match detector.detect() {
        Ok(manager) => {
            let migrator = get_converter(&manager, cli.path);

            migrator.convert_to_uv(
                cli.dry_run,
                cli.keep_current_data,
                cli.dependency_groups_strategy,
            );
        }
        Err(error) => {
            error!("{}", error);
            process::exit(1);
        }
    }

    process::exit(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli() {
        let cli = Cli::parse_from("migrate-to-uv --dry-run".split_whitespace());
        assert!(cli.dry_run);
    }
}
