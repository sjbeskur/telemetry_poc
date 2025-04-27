use std::collections::HashMap;

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

    pub fn increment_counter(&self, key: &str, value: u64) {
        let mut entry = self.metrics.entry(key.to_string()).or_insert(Metric {
            key: key.to_string(),
            value: MetricValue::Counter(0),
        });
        if let MetricValue::Counter(ref mut count) = entry.value {
            *count += value;
        }
    }

    pub fn set_gauge(&self, key: &str, value: f64) {
        self.metrics.insert(
            key.to_string(),
            Metric {
                key: key.to_string(),
                value: MetricValue::Gauge(value),
            },
        );
    }


    // pub fn add_table_row(&self, key: &str, row_index: &str, row_data: HashMap<String, MetricValue>) { // oid: &[u32]
    //     //let oid = oid.to_vec();        
    //     let mut entry = self.metrics.entry(key.to_string()).or_insert(Metric {
    //         key: key.to_string(),
    //         value: MetricValue::Table(vec![]),
    //     });
    //     if let MetricValue::Table(ref mut rows) = entry.value {
    //         if let Some(existing_row) = rows.iter_mut().find(|row| {
    //             row.get("index")
    //                 .map(|v| matches!(v, MetricValue::OctetString(s) if s == row_index))
    //                 .unwrap_or(false)
    //         }) {
    //             *existing_row = row_data;
    //         } else {
    //             let mut new_row = row_data;
    //             new_row.insert(
    //                 "index".to_string(),
    //                 MetricValue::OctetString(row_index.to_string()),
    //             );
    //             rows.push(new_row);
    //         }
    //     }
    // }
    
    pub fn get_all(&self) -> Vec<Metric> {
        self.metrics.iter().map(|entry| entry.value().clone()).collect()
    }

    pub fn get(&self, name: &str) -> Option<Metric> {
        self.metrics.get(name).map(|entry| entry.value().clone())
    }
}



// // Push metrics to endpoint
// async fn push_metrics(store: Arc<MetricStore>, client: Client, endpoint: String) {
//     let mut interval = time::interval(Duration::from_secs(5));
//     loop {
//         interval.tick().await;
//         let metrics = store.get_all();
//         match serde_json::to_string(&metrics) {
//             Ok(json) => {
//                 if let Err(e) = client
//                     .post(&endpoint)
//                     .header("Content-Type", "application/json")
//                     .body(json)
//                     .send()
//                     .await
//                 {
//                     eprintln!("Failed to push metrics: {}", e);
//                 }
//             }
//             Err(e) => eprintln!("Serialization error: {}", e),
//         }
//     }
// }

// Simulate application work
// async fn simulate_work(store: Arc<MetricStore>) {
//     let mut count = 0;
//     loop {
//         // Scalar metrics
//         store.increment_counter(&[2680, 1, 1], 1); // app.requests.total
//         store.set_gauge(&[2680, 1, 2], count as f64 * 0.5); // app.memory.mb

//         // Table: per-user metrics
//         let user_id = format!("user{}", count % 3); // Simulate 3 users
//         let row_data = HashMap::from([
//             (
//                 "reqs".to_string(),
//                 MetricValue::Counter((count % 10 + 1) as u64),
//             ),
//             (
//                 "lat".to_string(),
//                 MetricValue::Gauge((count % 5 + 1) as f64),
//             ),
//         ]);
//         store.add_table_row(&[2680, 2], &user_id, row_data);

//         count += 1;
//         time::sleep(Duration::from_secs(1)).await;
//     }
// }

// #[tokio::main]
// async fn main() -> std::io::Result<()> {
//     let store = Arc::new(MetricStore::new());
//     let client = Client::new();
//     let endpoint = "http://localhost:3000/telemetry".to_string();

//     // Spawn tasks
//     tokio::spawn(simulate_work(store.clone()));
//     tokio::spawn(push_metrics(store, client, endpoint));

//     // Keep the program running
//     tokio::signal::ctrl_c().await?;
//     Ok(())
// }