use std::collections::HashMap;

// Import Serialize and Deserialize macros from serde
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "t", content = "v")]
pub enum MetricValue {
    #[serde(rename = "cntr")]
    Counter(u64),         // increments 
    #[serde(rename = "gauge")]
    Gauge(f64),           // values may increase or decrease
    #[serde(rename = "hist")]
    Histogram(Vec<u64>),  // Buckets for simplicity
    #[serde(rename = "ticks")]
    TimeTicks(u64),       // track durations
    #[serde(rename = "int")]
    Integer(i32),         // status codes
    #[serde(rename = "string")]
    OctetString(String),  // string values
    //Table(Vec<HashMap<String, MetricValue>>), // Key-value pairs
}

// #[derive(Serialize, Deserialize, Clone, Debug)]
// pub struct OID{
//     pub name: String,
//     pub oid: String,
// }

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metric {
    #[serde(rename = "key")]
    pub key: String,
    #[serde(rename = "val")]
    pub value: MetricValue,
}