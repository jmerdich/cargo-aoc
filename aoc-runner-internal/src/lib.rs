extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use serde_derive::*;
use std::cmp::Ordering;
use std::error;
use std::fs;
use std::iter::FromIterator;
use std::ops::Deref;
use std::ops::DerefMut;
use std::str::FromStr;

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Day(pub u8);

impl FromStr for Day {
    type Err = String;

    fn from_str(day: &str) -> Result<Self, Self::Err> {
        let slice = if day.len() < 4 || &day[..3] != "day" {
            day
        } else {
            &day[3..]
        };

        slice
            .parse()
            .map_err(|e| format!("Failed to parse {}: {:?}", day, e))
            .and_then(|d| {
                if d == 0 || d > 25 {
                    Err(format!("day {} is not between 0 and 25", d))
                } else {
                    Ok(Day(d))
                }
            })
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Copy, Clone, Serialize, Deserialize, Ord, PartialOrd)]
pub struct Part(pub u8);

impl FromStr for Part {
    type Err = String;

    fn from_str(part: &str) -> Result<Self, Self::Err> {
        Ok(match part {
            "part1" | "1" => Part(1),
            "part2" | "2" => Part(2),
            _ => return Err(format!("Failed to parse part: {}", part)),
        })
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct DayPart {
    pub day: Day,
    pub part: Part,
    pub name: Option<String>,
}

impl DayPart {
    pub fn without_name(&self) -> DayPart {
        DayPart {
            name: None,
            day: self.day,
            part: self.part,
        }
    }
}

impl PartialOrd for DayPart {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DayPart {
    fn cmp(&self, other: &Self) -> Ordering {
        self.day
            .cmp(&other.day)
            .then(self.part.cmp(&other.part))
            .then(self.name.cmp(&other.name))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DayParts {
    pub year: u32,
    parts: Vec<DayPart>,
}

impl DayParts {
    pub fn save(&self) -> Result<(), Box<dyn error::Error>> {
        let mut target_directory: std::path::PathBuf = "target".into();
        let proc_dir_path: std::path::PathBuf = env!("PROC_OUT_DIR").into();
        for potential_path in proc_dir_path.ancestors() {
            // There isn't a good way to get the target root dir from a proc-macro.
            // But a build script gets its own OUT_DIR in the target dir, so pass that
            // so the proc-macro can see it and search upwards from there.
            // This is a hack, but so is the whole saving-and-loading-during-build thing.
            if potential_path.file_name() == Some("target".as_ref()) {
                target_directory = potential_path.into();
                break;
            }
        }
        if !target_directory.join("aoc").exists() {
            fs::create_dir(target_directory.join("aoc"))?;
        }
        let crate_name = std::env::var("CARGO_CRATE_NAME").expect("Cannot determine crate name");
        let f =
            fs::File::create(target_directory.join(format!("aoc/completed_{}.json", crate_name)))?;
        serde_json::to_writer_pretty(f, &self)?;

        // Still write to old filename, in case our versions are mismatched.
        let f = fs::File::create(target_directory.join("aoc/completed.json"))?;

        serde_json::to_writer_pretty(f, &self)?;

        Ok(())
    }

    pub fn load(
        crate_name: String,
        root_dir: Option<std::path::PathBuf>,
    ) -> Result<Self, Box<dyn error::Error>> {
        let root_dir = root_dir.unwrap_or_else(|| "target".into());
        let mut path = root_dir
            .join("aoc")
            .join(format!("completed_{}.json", crate_name));
        let mut opt_f = fs::File::open(&path);

        if let Err(err) = &opt_f {
            if err.kind() == std::io::ErrorKind::NotFound {
                // If missing, get old filename, in case our versions are mismatched
                path.pop();
                path.push("completed.json");
                opt_f = fs::File::open(path);
            }
        }

        Ok(serde_json::from_reader(opt_f?)?)
    }
}

impl Deref for DayParts {
    type Target = [DayPart];

    fn deref(&self) -> &[DayPart] {
        &self.parts
    }
}

impl DerefMut for DayParts {
    fn deref_mut(&mut self) -> &mut [DayPart] {
        &mut self.parts
    }
}

pub struct DayPartsBuilder {
    parts: Vec<DayPart>,
}

impl DayPartsBuilder {
    pub fn with_year(self, year: u32) -> DayParts {
        DayParts {
            year,
            parts: self.parts,
        }
    }
}

impl FromIterator<DayPart> for DayPartsBuilder {
    fn from_iter<T: IntoIterator<Item = DayPart>>(iter: T) -> Self {
        let parts = iter.into_iter().collect();
        DayPartsBuilder { parts }
    }
}
