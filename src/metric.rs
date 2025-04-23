// Import Serialize and Deserialize macros from serde
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<u64>), // Buckets for simplicity
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OID{
    pub name: String,
    pub oid: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metric {
    pub name: String,
    pub value: MetricValue,
}