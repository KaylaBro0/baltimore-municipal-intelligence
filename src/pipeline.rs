use crate::{
    analysis::QueryConfig,
    concurrency::parallel_map,
    models::{Incident, JoinedRecord, Settlement},
};
use std::collections::HashMap;

/// Joins the officer-cost and crime datasets by district when a district exists,
/// otherwise it falls back to the configured district filter and retains the
/// cross-dataset relation in a bounded, zero-copy way.
pub fn join<'a>(
    settlements: &'a [Settlement],
    incidents: &'a [Incident],
    config: &QueryConfig,
) -> Vec<JoinedRecord<'a>> {
    let mut index: HashMap<String, Vec<&Incident>> = HashMap::new();

    for incident in incidents {
        if !config.matches_district(&incident.district) {
            continue;
        }
        index
            .entry(incident.district.clone())
            .or_default()
            .push(incident);
    }

    let partials = parallel_map(settlements, |settlement| {
        if settlement.total_cost() < config.min_settlement_usd() {
            return Vec::new();
        }

        let district_keys = match settlement.district.as_deref() {
            Some(district) => vec![district.to_string()],
            None => {
                if let Some(district) = config.district_name() {
                    vec![district.to_string()]
                } else {
                    index.keys().cloned().collect()
                }
            }
        };

        district_keys
            .iter()
            .filter_map(|district| index.get(district))
            .flatten()
            .map(|incident| JoinedRecord {
                settlement,
                incident,
            })
            .collect::<Vec<_>>()
    });

    partials.into_iter().flatten().collect()
}
