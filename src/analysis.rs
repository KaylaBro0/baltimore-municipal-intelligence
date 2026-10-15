use crate::models::{Incident, JoinedRecord, Metric, Report, Settlement};
use chrono::{Datelike, NaiveDate, NaiveDateTime};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Default)]
pub struct QueryConfig {
    district: Option<String>,
    min_settlement_usd: f64,
}

pub struct QueryBuilder {
    district: Option<String>,
    min_settlement_usd: f64,
}

impl QueryConfig {
    pub fn builder() -> QueryBuilder {
        QueryBuilder {
            district: None,
            min_settlement_usd: 0.0,
        }
    }

    pub fn matches_district(&self, district: &str) -> bool {
        self.district
            .as_deref()
            .map_or(true, |filter| filter.eq_ignore_ascii_case(district))
    }

    pub fn district_name(&self) -> Option<&str> {
        self.district.as_deref()
    }

    pub fn min_settlement_usd(&self) -> f64 {
        self.min_settlement_usd
    }
}

impl QueryBuilder {
    pub fn district(mut self, district: Option<String>) -> Self {
        self.district = district;
        self
    }

    pub fn settlement_threshold(mut self, threshold: f64) -> Self {
        self.min_settlement_usd = threshold;
        self
    }

    pub fn build(self) -> QueryConfig {
        QueryConfig {
            district: self.district,
            min_settlement_usd: self.min_settlement_usd,
        }
    }
}

trait AnalysisStrategy {
    fn calculate(
        &self,
        settlements: &[Settlement],
        incidents: &[Incident],
        joined: &[JoinedRecord<'_>],
        config: &QueryConfig,
    ) -> Vec<Metric>;
}

struct DistrictCost;
struct UnitHotspots;
struct Timeline;
struct Recurrence;
struct Risk;

fn parse_year(raw: &str) -> Option<i32> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    for fmt in [
        "%Y-%m-%d",
        "%m/%d/%Y %I:%M:%S %p",
        "%m/%d/%Y %H:%M:%S",
        "%m/%d/%Y",
        "%Y/%m/%d",
    ] {
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, fmt) {
            return Some(date.year());
        }
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, fmt) {
            return Some(dt.year());
        }
    }

    None
}

fn incident_counts_by_district(incidents: &[Incident], config: &QueryConfig) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for incident in incidents {
        if !config.matches_district(&incident.district) {
            continue;
        }
        *counts.entry(incident.district.clone()).or_default() += 1;
    }
    counts
}

fn district_cost_weighting(
    settlements: &[Settlement],
    incidents: &[Incident],
    config: &QueryConfig,
) -> HashMap<String, f64> {
    let counts = incident_counts_by_district(incidents, config);
    let total_incidents = counts.values().sum::<usize>() as f64;
    let total_settlement: f64 = settlements
        .iter()
        .filter(|row| row.total_cost() >= config.min_settlement_usd())
        .map(|row| row.total_cost())
        .sum();

    let mut weighted: HashMap<String, f64> = HashMap::new();
    if total_incidents > 0.0 && total_settlement > 0.0 {
        for (district, count) in counts {
            let share = total_settlement * (count as f64 / total_incidents);
            weighted.insert(district, share);
        }
    }

    weighted
}

impl AnalysisStrategy for DistrictCost {
    fn calculate(
        &self,
        settlements: &[Settlement],
        incidents: &[Incident],
        _: &[JoinedRecord<'_>],
        config: &QueryConfig,
    ) -> Vec<Metric> {
        let counts = incident_counts_by_district(incidents, config);
        let weighted_costs = district_cost_weighting(settlements, incidents, config);

        let mut metrics: Vec<_> = counts
            .into_iter()
            .map(|(district, count)| {
                let cost = weighted_costs.get(&district).copied().unwrap_or(0.0);
                Metric {
                    label: district.clone(),
                    value: cost,
                    detail: format!("${cost:.0} modeled cost; {count} incidents"),
                }
            })
            .collect();

        metrics.sort_by(|a, b| b.value.total_cmp(&a.value));
        metrics
    }
}

impl AnalysisStrategy for UnitHotspots {
    fn calculate(
        &self,
        settlements: &[Settlement],
        incidents: &[Incident],
        joined: &[JoinedRecord<'_>],
        config: &QueryConfig,
    ) -> Vec<Metric> {
        let mut groups: HashMap<String, (usize, f64, HashSet<String>)> = HashMap::new();
        let district_costs = district_cost_weighting(settlements, incidents, config);
        let incident_counts = incident_counts_by_district(incidents, config);

        if joined.is_empty() {
            for incident in incidents {
                if !config.matches_district(&incident.district) {
                    continue;
                }
                let label = format!(
                    "{} / {}",
                    incident
                        .neighborhood
                        .clone()
                        .unwrap_or_else(|| incident.district.clone()),
                    incident.description.clone()
                );
                let group = groups.entry(label).or_default();
                group.0 += 1;
                let district_cost = district_costs.get(&incident.district).copied().unwrap_or(0.0);
                let district_count = incident_counts.get(&incident.district).copied().unwrap_or(1) as f64;
                group.1 += if district_count > 0.0 {
                    district_cost / district_count
                } else {
                    0.0
                };
                group.2.insert(
                    incident
                        .incident_id
                        .clone()
                        .or_else(|| incident.row_id.clone())
                        .unwrap_or_else(|| incident.crime_code.clone()),
                );
            }
        } else {
            for row in joined {
                if row.settlement.total_cost() < config.min_settlement_usd() {
                    continue;
                }

                let label = format!(
                    "{} / {}",
                    row.incident
                        .neighborhood
                        .clone()
                        .unwrap_or_else(|| row.incident.district.clone()),
                    row.incident.description.clone()
                );
                let group = groups.entry(label).or_default();
                group.0 += 1;
                group.1 += row.settlement.total_cost();
                group.2.insert(
                    row.incident
                        .incident_id
                        .clone()
                        .or_else(|| row.incident.row_id.clone())
                        .unwrap_or_else(|| row.incident.crime_code.clone()),
                );
            }
        }

        let mut metrics: Vec<_> = groups
            .into_iter()
            .map(|(label, (pairs, payout, incident_ids))| Metric {
                label,
                value: pairs as f64,
                detail: format!(
                    "{pairs} flagged records, {} distinct objects, ${payout:.0} modeled cost",
                    incident_ids.len()
                ),
            })
            .collect();

        metrics.sort_by(|a, b| b.value.total_cmp(&a.value));
        metrics.truncate(8);
        metrics
    }
}

impl AnalysisStrategy for Timeline {
    fn calculate(
        &self,
        settlements: &[Settlement],
        incidents: &[Incident],
        _: &[JoinedRecord<'_>],
        config: &QueryConfig,
    ) -> Vec<Metric> {
        let mut years: BTreeMap<i32, (usize, usize)> = BTreeMap::new();

        for settlement in settlements {
            if settlement.total_cost() < config.min_settlement_usd() {
                continue;
            }
            if let Some(district) = settlement.district.as_deref() {
                if !config.matches_district(district) {
                    continue;
                }
            }
            let year = settlement
                .report_year
                .or_else(|| settlement.settlement_date.as_deref().and_then(parse_year))
                .or_else(|| settlement.image_capture_date.as_deref().and_then(parse_year))
                .unwrap_or_default();
            let entry = years.entry(year).or_default();
            entry.0 += 1;
        }

        for incident in incidents.iter().filter(|row| config.matches_district(&row.district)) {
            let year = parse_year(&incident.crime_date_time).unwrap_or_default();
            let entry = years.entry(year).or_default();
            entry.1 += 1;
        }

        years
            .into_iter()
            .map(|(year, (settlements_count, incidents_count))| Metric {
                label: year.to_string(),
                value: settlements_count as f64,
                detail: format!(
                    "{settlements_count} settlement records; {incidents_count} crime records"
                ),
            })
            .collect()
    }
}

impl AnalysisStrategy for Recurrence {
    fn calculate(
        &self,
        settlements: &[Settlement],
        _incidents: &[Incident],
        _: &[JoinedRecord<'_>],
        config: &QueryConfig,
    ) -> Vec<Metric> {
        let mut counts: HashMap<String, (usize, f64)> = HashMap::new();

        for settlement in settlements {
            if settlement.total_cost() < config.min_settlement_usd() {
                continue;
            }
            if let Some(district) = settlement.district.as_deref() {
                if !config.matches_district(district) {
                    continue;
                }
            }
            let entry = counts.entry(settlement.officer_name.clone()).or_default();
            entry.0 += 1;
            entry.1 += settlement.total_cost();
        }

        let total_cost: f64 = counts.values().map(|value| value.1).sum();
        let mean_cost = if counts.is_empty() {
            0.0
        } else {
            total_cost / counts.len() as f64
        };

        let mut metrics: Vec<_> = counts
            .into_iter()
            .map(|(officer, (records, cost))| Metric {
                label: officer,
                value: cost,
                detail: format!(
                    "{records} settlement records; ${cost:.0} total; {:.1}x mean officer cost",
                    if mean_cost > 0.0 { cost / mean_cost } else { 0.0 }
                ),
            })
            .collect();

        metrics.sort_by(|a, b| b.value.total_cmp(&a.value));
        metrics.truncate(10);
        metrics
    }
}

impl AnalysisStrategy for Risk {
    fn calculate(
        &self,
        settlements: &[Settlement],
        incidents: &[Incident],
        _: &[JoinedRecord<'_>],
        config: &QueryConfig,
    ) -> Vec<Metric> {
        let counts = incident_counts_by_district(incidents, config);
        let weighted_costs = district_cost_weighting(settlements, incidents, config);
        let mut districts: HashMap<String, (f64, usize)> = HashMap::new();

        for (district, count) in counts {
            let cost = weighted_costs.get(&district).copied().unwrap_or(0.0);
            districts.insert(district, (cost, count));
        }

        let max_cost = districts.values().map(|value| value.0).fold(0.0, f64::max).max(1.0);
        let max_incidents = districts.values().map(|value| value.1).max().unwrap_or(1).max(1) as f64;

        let mut metrics: Vec<_> = districts
            .into_iter()
            .map(|(district, (cost, incidents))| {
                let score = 100.0 * (0.6 * cost / max_cost + 0.4 * incidents as f64 / max_incidents);
                Metric {
                    label: district,
                    value: score,
                    detail: format!("{score:.1}/100; ${cost:.0} modeled cost, {incidents} incidents"),
                }
            })
            .collect();

        metrics.sort_by(|a, b| b.value.total_cmp(&a.value));
        metrics
    }
}

pub fn run_all(
    settlements: &[Settlement],
    incidents: &[Incident],
    joined: &[JoinedRecord<'_>],
    config: &QueryConfig,
) -> Report {
    let strategies: [&dyn AnalysisStrategy; 5] = [
        &DistrictCost,
        &UnitHotspots,
        &Timeline,
        &Recurrence,
        &Risk,
    ];

    Report {
        q1: strategies[0].calculate(settlements, incidents, joined, config),
        q2: strategies[1].calculate(settlements, incidents, joined, config),
        q3: strategies[2].calculate(settlements, incidents, joined, config),
        q4: strategies[3].calculate(settlements, incidents, joined, config),
        q5: strategies[4].calculate(settlements, incidents, joined, config),
        caveat: "These analyses combine the uploaded NIBRS crime data with officer settlement costs. Results are descriptive summaries for operational reviews and should be validated against the final source records and any district mapping rules before making policy decisions.".into(),
    }
}
