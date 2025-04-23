use dashmap::DashMap;
use crate::metric::{Metric, MetricValue};

// Thread-safe metric store

pub struct MetricStore {
    metrics: DashMap<String, Metric>,
}

impl MetricStore {
    pub fn new() -> Self {
        MetricStore {
            metrics: DashMap::new(),
        }
    }

    pub fn increment_counter(&self, name: &str, value: u64) {
        let mut entry = self.metrics.entry(name.to_string()).or_insert(Metric {
            name: name.to_string(),
            value: MetricValue::Counter(0),
        });
        if let MetricValue::Counter(ref mut count) = entry.value {
            *count += value;
        }
    }

    pub fn set_gauge(&self, name: &str, value: f64) {
        self.metrics.insert(
            name.to_string(),
            Metric {
                name: name.to_string(),
                value: MetricValue::Gauge(value),
            },
        );
    }

    pub fn get_all(&self) -> Vec<Metric> {
        self.metrics.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get(&self, name: &str) -> Option<Metric> {
        self.metrics.get(name).map(|entry| entry.value().clone())
    }
}