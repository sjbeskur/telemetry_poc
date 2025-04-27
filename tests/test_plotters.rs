use std::sync::Arc;

use plotters::{chart::ChartBuilder, prelude::{BitMapBackend, IntoDrawingArea, PathElement}, series::LineSeries, style::Color};
use plotters::style::{IntoFont, BLUE, RED, WHITE,GREEN, BLACK};
use plotters::style::full_palette::{ORANGE, PURPLE};

use telemetry_poc::MetricStore;


#[test]
fn test_plotters(){
    let store = MetricStore::new();
    for i in 0..10 {
        store.increment_counter("test.counter", 1);
    }

}

use chrono::Utc;
use dashmap::DashMap;
use plotters::prelude::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{self, Duration};

// Metric value types
#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "t", content = "v")]
enum MetricValue {
    Counter(u64),
    Gauge(f64),
}

#[derive(Serialize, Deserialize, Clone)]
struct Metric {
    oid: Vec<u32>, // Post-PEN OID, e.g., [2680, 1, 1]
    value: MetricValue,
}

#[derive(Clone)]
struct MetricHistory {
    oid: Vec<u32>,
    values: Vec<(i64, f64)>, // (timestamp_ms, value)
}

#[derive(Serialize, Deserialize, Clone)]
struct Table {
    oid: Vec<u32>, // Table OID, e.g., [2680, 2]
    rows: HashMap<String, HashMap<String, MetricValue>>, // Row index -> column data
}

#[derive(Clone)]
struct TableHistory {
    oid: Vec<u32>,
    rows: HashMap<String, Vec<(i64, f64)>>, // Row index -> (timestamp_ms, value) for a column
}

// Thread-safe metric store
struct MetricStore {
    metrics: DashMap<Vec<u32>, Metric>,
    metric_histories: DashMap<Vec<u32>, MetricHistory>,
    tables: DashMap<Vec<u32>, Table>,
    table_histories: DashMap<Vec<u32>, TableHistory>,
}

impl MetricStore {
    fn new() -> Self {
        MetricStore {
            metrics: DashMap::new(),
            metric_histories: DashMap::new(),
            tables: DashMap::new(),
            table_histories: DashMap::new(),
        }
    }

    fn increment_counter(&self, oid: &[u32], value: u64) {
        let oid = oid.to_vec();
        let entry = self.metrics.entry(oid.clone()).or_insert(Metric {
            oid: oid.clone(),
            value: MetricValue::Counter(0),
        });
        let new_value = if let MetricValue::Counter(ref mut count) = entry.value {
            *count += value;
            *count as f64
        } else {
            0.0
        };
        let history = self.metric_histories.entry(oid.clone()).or_insert(MetricHistory {
            oid,
            values: Vec::new(),
        });
        history.values.push((Utc::now().timestamp_millis(), new_value));
        // Limit history to last 100 points
        if history.values.len() > 100 {
            history.values.drain(0..history.values.len() - 100);
        }
    }

    fn set_gauge(&self, oid: &[u32], value: f64) {
        let oid = oid.to_vec();
        self.metrics.insert(
            oid.clone(),
            Metric {
                oid: oid.clone(),
                value: MetricValue::Gauge(value),
            },
        );
        let history = self.metric_histories.entry(oid.clone()).or_insert(MetricHistory {
            oid,
            values: Vec::new(),
        });
        history.values.push((Utc::now().timestamp_millis(), value));
        if history.values.len() > 100 {
            history.values.drain(0..history.values.len() - 100);
        }
    }

    fn add_table_row(&self, oid: &[u32], row_index: &str, row_data: HashMap<String, MetricValue>) {
        let oid = oid.to_vec();
        let table = self.tables.entry(oid.clone()).or_insert(Table {
            oid: oid.clone(),
            rows: HashMap::new(),
        });
        table.rows.insert(row_index.to_string(), row_data.clone());

        // Update table history for 'reqs' column
        let table_history = self.table_histories.entry(oid.clone()).or_insert(TableHistory {
            oid,
            rows: HashMap::new(),
        });
        let row_history = table_history
            .rows
            .entry(row_index.to_string())
            .or_insert(Vec::new());
        if let Some(MetricValue::Counter(value)) = row_data.get("reqs") {
            row_history.push((Utc::now().timestamp_millis(), *value as f64));
            if row_history.len() > 100 {
                row_history.drain(0..row_history.len() - 100);
            }
        }
    }

    fn get_all(&self) -> (Vec<Metric>, Vec<Table>) {
        let metrics = self.metrics.iter().map(|entry| entry.value().clone()).collect();
        let tables = self.tables.iter().map(|entry| entry.value().clone()).collect();
        (metrics, tables)
    }

    fn get_histories(&self) -> (Vec<MetricHistory>, Vec<TableHistory>) {
        let metrics = self
            .metric_histories
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        let tables = self
            .table_histories
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        (metrics, tables)
    }
}

// Push metrics to REST endpoint
async fn push_metrics(store: Arc<MetricStore>, client: Client, endpoint: String) {
    let mut interval = time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        let (metrics, tables) = store.get_all();
        let payload = serde_json::json!({ "metrics": metrics, "tables": tables });
        match serde_json::to_string(&payload) {
            Ok(json) => {
                if let Err(e) = client
                    .post(&endpoint)
                    .header("Content-Type", "application/json")
                    .body(json)
                    .send()
                    .await
                {
                    eprintln!("Failed to push metrics: {}", e);
                }
            }
            Err(e) => eprintln!("Serialization error: {}", e),
        }
    }
}

// Plot metrics using Plotters
async fn plot_metrics(store: Arc<MetricStore>) {
    let mut interval = time::interval(Duration::from_secs(5));
    loop {
        interval.tick().await;
        let (metric_histories, table_histories) = store.get_histories();

        // Setup Plotters
        let root = BitMapBackend::new("telemetry.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let (min_time, max_time, min_value, max_value) = metric_histories
            .iter()
            .flat_map(|h| h.values.iter())
            .chain(
                table_histories
                    .iter()
                    .flat_map(|t| t.rows.values().flat_map(|v| v.iter())),
            )
            .fold(
                (i64::MAX, i64::MIN, f64::MAX, f64::MIN),
                |(min_t, max_t, min_v, max_v), &(t, v)| {
                    (min_t.min(t), max_t.max(t), min_v.min(v), max_v.max(v))
                },
            );
        if min_time == i64::MAX || max_time == i64::MIN {
            continue; // No data yet
        }

        let mut chart = ChartBuilder::on(&root)
            .caption("Telemetry Metrics", ("sans-serif", 20).into_font())
            .x_label_area_size(40)
            .y_label_area_size(40)
            .margin(10)
            .build_cartesian_2d(
                (min_time as f64)..(max_time as f64),
                (min_value * 0.9)..(max_value * 1.1),
            )
            .unwrap();

        chart
            .configure_mesh()
            .x_desc("Time (ms)")
            .y_desc("Value")
            .draw()
            .unwrap();

        // Plot scalar metrics
        let colors = [RED, BLUE];
        for (i, history) in metric_histories.iter().enumerate() {
            let label = format!("{:?}", history.oid);
            chart
                .draw_series(LineSeries::new(
                    history.values.iter().map(|&(t, v)| (t as f64, v)),
                    colors[i % colors.len()].stroke_width(2),
                ))
                .unwrap()
                .label(label)
                .legend(move |(x, y)| {
                    PathElement::new(
                        vec![(x, y), (x + 20, y)],
                        colors[i % colors.len()].stroke_width(2),
                    )
                });
        }

        // Plot table 'reqs' column
        let table_colors = [GREEN, PURPLE, ORANGE];
        for table in table_histories.iter() {
            for (i, (row_index, values)) in table.rows.iter().enumerate() {
                let label = format!("{:?}[{}].reqs", table.oid, row_index);
                chart
                    .draw_series(LineSeries::new(
                        values.iter().map(|&(t, v)| (t as f64, v)),
                        table_colors[i % table_colors.len()].stroke_width(2),
                    ))
                    .unwrap()
                    .label(label)
                    .legend(move |(x, y)| {
                        PathElement::new(
                            vec![(x, y), (x + 20, y)],
                            table_colors[i % table_colors.len()].stroke_width(2),
                        )
                    });
            }
        }

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();

        root.present().unwrap();
    }
}

// Simulate application work
async fn simulate_work(store: Arc<MetricStore>) {
    let mut count = 0;
    loop {
        // Scalar metrics
        store.increment_counter(&[2680, 1, 1], 1); // app.requests.total
        store.set_gauge(&[2680, 1, 2], count as f64 * 0.5); // app.memory.mb

        // Table: per-user metrics
        let user_id = format!("user{}", count % 3); // Simulate 3 users
        let row_data = HashMap::from([
            ("reqs".to_string(), MetricValue::Counter((count % 10 + 1) as u64)),
            ("lat".to_string(), MetricValue::Gauge((count % 5 + 1) as f64)),
        ]);
        store.add_table_row(&[2680, 2], &user_id, row_data);

        count += 1;
        time::sleep(Duration::from_secs(1)).await;
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let store = Arc::new(MetricStore::new());
    let client = Client::new();
    let endpoint = "http://localhost:3000/telemetry".to_string();

    // Spawn tasks
    tokio::spawn(simulate_work(store.clone()));
    tokio::spawn(push_metrics(store.clone(), client, endpoint));
    tokio::spawn(plot_metrics(store));

    // Keep the program running
    tokio::signal::ctrl_c().await?;
    Ok(())
}