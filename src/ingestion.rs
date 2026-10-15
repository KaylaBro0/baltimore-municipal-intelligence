use crate::models::{Incident, Settlement};
use anyhow::{Context, Result};
use rayon::join;
use std::{fs, path::{Path, PathBuf}, process::Command};

#[derive(Debug)]
pub struct Sources {
    pub settlements: String,
    pub incidents: String,
}

fn fetch(url: &str) -> Result<Vec<u8>> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            "20",
            url,
        ])
        .output()
        .with_context(|| "could not start curl for optional live CSV download")?;
    anyhow::ensure!(
        output.status.success(),
        "live download failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

fn resolve_dataset_path(base_dir: &Path, preferred_names: &[&str], fallback_names: &[&str]) -> PathBuf {
    for name in preferred_names {
        let candidate = base_dir.join(name);
        if candidate.exists() {
            return candidate;
        }
    }
    for name in fallback_names {
        let candidate = base_dir.join(name);
        if candidate.exists() {
            return candidate;
        }
    }
    base_dir.join(preferred_names[0])
}

fn load_dataset<T: for<'de> serde::Deserialize<'de>>(
    path: &Path,
    url: Option<&str>,
    label: &str,
) -> Result<(Vec<T>, String)> {
    if let Some(url) = url {
        if let Ok(bytes) = fetch(url) {
            if !bytes.is_empty() {
                if let Ok(records) = parse_csv(&bytes, label) {
                    if !records.is_empty() {
                        return Ok((records, format!("live URL {url}")));
                    }
                }
            }
        }
    }

    let bytes = fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    let records = parse_csv(&bytes, label)?;
    Ok((records, format!("local {}", path.display())))
}

fn parse_csv<T: for<'de> serde::Deserialize<'de>>(
    bytes: &[u8],
    label: &str,
) -> Result<Vec<T>> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .flexible(true)
        .from_reader(bytes);
    reader
        .deserialize()
        .collect::<std::result::Result<Vec<_>, _>>()
        .with_context(|| {
            format!("parsing {label}; expected the documented normalized CSV columns")
        })
}

pub fn load_all(
    dir: &Path,
    settlement_url: Option<&str>,
    incident_url: Option<&str>,
) -> Result<(Vec<Settlement>, Vec<Incident>, Sources)> {
    let settlement_path = resolve_dataset_path(
        dir,
        &["gttf_tracker_officer_costs.csv", "gttf.csv"],
        &["data/gttf.csv", "data/gttf_tracker_officer_costs.csv"],
    );
    let incident_path = resolve_dataset_path(
        dir,
        &["NIBRS_GroupA_Crime_Data_1100291637803527672.csv", "incidents.csv"],
        &["data/incidents.csv", "data/NIBRS_GroupA_Crime_Data_1100291637803527672.csv"],
    );

    let (settlements_result, incidents_result) = join(
        || load_dataset::<Settlement>(&settlement_path, settlement_url, "settlements"),
        || load_dataset::<Incident>(&incident_path, incident_url, "incidents"),
    );

    let (settlements, settlements_source) = settlements_result?;
    let (incidents, incidents_source) = incidents_result?;

    anyhow::ensure!(
        !settlements.is_empty() && !incidents.is_empty(),
        "input datasets must each contain at least one record"
    );

    Ok((
        settlements,
        incidents,
        Sources {
            settlements: settlements_source,
            incidents: incidents_source,
        },
    ))
}
