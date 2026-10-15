mod analysis;
mod concurrency;
mod ingestion;
mod models;
mod pipeline;
mod ui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "bmie",
    about = "Baltimore Municipal Intelligence Engine (offline-first demo)"
)]
struct Args {
    /// Directory containing the uploaded CSV inputs. Defaults to the project root.
    #[arg(long, default_value = ".")]
    data_dir: PathBuf,
    /// Filter results to a district (for example Western or Central)
    #[arg(long)]
    district: Option<String>,
    /// Optional URLs of the officer-cost dataset; local assets remain the fallback
    #[arg(long)]
    settlements_url: Option<String>,
    /// Optional URLs of the NIBRS crime dataset; local assets remain the fallback
    #[arg(long)]
    incidents_url: Option<String>,
    /// Optional settlement threshold to ignore very small payouts
    #[arg(long, default_value = "0")]
    min_settlement_usd: f64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let Args {
        data_dir,
        district,
        settlements_url,
        incidents_url,
        min_settlement_usd,
    } = args;

    let (settlements, incidents, sources, report) = tokio::task::spawn_blocking(move || {
        let (settlements, incidents, sources) = ingestion::load_all(
            &data_dir,
            settlements_url.as_deref(),
            incidents_url.as_deref(),
        )?;
        let config = analysis::QueryConfig::builder()
            .district(district)
            .settlement_threshold(min_settlement_usd)
            .build();
        let joined = pipeline::join(&settlements, &incidents, &config);
        let report = analysis::run_all(&settlements, &incidents, &joined, &config);

        Ok::<_, anyhow::Error>((settlements, incidents, sources, report))
    })
    .await??;

    ui::render(&report, &sources, settlements.len(), incidents.len());
    Ok(())
}
