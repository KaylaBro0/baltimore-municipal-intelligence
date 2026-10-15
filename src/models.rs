use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Settlement {
    #[serde(default)]
    pub officer_id: Option<String>,
    #[serde(rename = "officer_name", alias = "officer_name")]
    pub officer_name: String,
    #[serde(default, rename = "cumulative_settlement_cost_usd")]
    pub cumulative_settlement_cost_usd: f64,
    #[serde(default, rename = "amount_usd")]
    pub amount_usd: f64,
    #[serde(default)]
    pub settlement_date: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default, alias = "New_District", alias = "district")]
    pub district: Option<String>,
    #[serde(default)]
    pub case_type: Option<String>,
    #[serde(default)]
    pub report_year: Option<i32>,
    #[serde(default)]
    pub discipline_year: Option<i32>,
    #[serde(default)]
    pub tracker_cases_settled: Option<u32>,
    #[serde(default)]
    pub tracker_total_paid_usd: Option<f64>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default, rename = "image_capture_date")]
    pub image_capture_date: Option<String>,
}

impl Settlement {
    pub fn total_cost(&self) -> f64 {
        if self.cumulative_settlement_cost_usd > 0.0 {
            self.cumulative_settlement_cost_usd
        } else {
            self.amount_usd
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Incident {
    #[serde(default, rename = "RowID")]
    pub row_id: Option<String>,
    #[serde(default)]
    pub cc_number: Option<String>,
    #[serde(rename = "CrimeDateTime", alias = "incident_date")]
    pub crime_date_time: String,
    #[serde(rename = "CrimeCode", alias = "crime_type")]
    pub crime_code: String,
    #[serde(rename = "Description", alias = "description")]
    pub description: String,
    #[serde(default, rename = "Inside_Outside")]
    pub inside_outside: Option<String>,
    #[serde(default)]
    pub weapon: Option<String>,
    #[serde(default)]
    pub shooting: Option<String>,
    #[serde(default)]
    pub post: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub age: Option<i32>,
    #[serde(default)]
    pub race: Option<String>,
    #[serde(default)]
    pub ethnicity: Option<String>,
    #[serde(default)]
    pub location: Option<String>,
    #[serde(default, rename = "Old_District")]
    pub old_district: Option<String>,
    #[serde(default, rename = "New_District", alias = "district")]
    pub district: String,
    #[serde(default)]
    pub neighborhood: Option<String>,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
    #[serde(default)]
    pub geo_location: Option<String>,
    #[serde(default)]
    pub premise_type: Option<String>,
    #[serde(default)]
    pub total_incidents: Option<i32>,
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub incident_id: Option<String>,
    #[serde(default)]
    pub officer_id: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub shift: Option<String>,
}

#[derive(Debug, Clone)]
pub struct JoinedRecord<'a> {
    pub settlement: &'a Settlement,
    pub incident: &'a Incident,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub label: String,
    pub value: f64,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct Report {
    pub q1: Vec<Metric>,
    pub q2: Vec<Metric>,
    pub q3: Vec<Metric>,
    pub q4: Vec<Metric>,
    pub q5: Vec<Metric>,
    pub caveat: String,
}
