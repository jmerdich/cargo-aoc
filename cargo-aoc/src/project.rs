use aoc_runner_internal::DayParts;
use camino::Utf8PathBuf;
use cargo_metadata::{MetadataCommand, Package};
use std::error;
use std::process;

pub struct ProjectManager {
    pub name: String,
    pub slug: String,
    pub root_target_dir: camino::Utf8PathBuf,
    pub crate_dir: camino::Utf8PathBuf,
}

impl ProjectManager {
    pub fn new() -> Result<ProjectManager, Box<dyn error::Error>> {
        let metadata = MetadataCommand::new().exec()?;

        // First filter the (usually many) dependencies to just the workspace
        // members
        let workspace_pkgs: Vec<Package> = metadata
            .workspace_members
            .iter()
            .map(|id| metadata[id].clone())
            .collect();

        let mut cur_package = None;
        assert!(!workspace_pkgs.is_empty());
        if let Some(root_pkg) = metadata.root_package() {
            cur_package = Some(root_pkg.clone());
        } else {
            let cur_dir = std::env::current_dir()?;
            // Determine which we care about by checking which directory we're currently in
            // and seeing if it's a subdir of where the Cargo.toml manifest is.
            for pkg in workspace_pkgs {
                if cur_dir.starts_with(pkg.manifest_path.parent().unwrap()) {
                    cur_package = Some(pkg);
                    break;
                }
            }
        }

        let pkg = cur_package.ok_or("Unable to determine current crate")?;
        let crate_slug = pkg.name.replace('-', "_");

        Ok(ProjectManager {
            name: pkg.name,
            slug: crate_slug,
            root_target_dir: metadata.target_directory,
            crate_dir: pkg.manifest_path.parent().unwrap().into(),
        })
    }

    pub fn build_project(&self) -> Result<DayParts, Box<dyn error::Error>> {
        let args = vec!["check", "--color=always"];

        let status = process::Command::new("cargo").args(&args).spawn()?.wait()?;

        if !status.success() {
            return Err(format!(
                "cargo build failed with code {}",
                status.code().unwrap_or(-1)
            )
            .into());
        }

        DayParts::load(self.slug.clone(), Some(self.root_target_dir.clone().into()))
    }

    pub fn input_file_for(&self, year: u32, day: aoc_runner_internal::Day) -> Utf8PathBuf {
        self.crate_dir
            .join("input")
            .join(year.to_string())
            .join(format!("day{}.txt", day.0))
    }
}
