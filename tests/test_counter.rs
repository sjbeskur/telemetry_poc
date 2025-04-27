use std::collections::HashMap;

use telemetry_poc::*;


#[test]
fn test_counter() {

    let store = MetricStore::new();
    for i in 0..10 {
        store.increment_counter("test.counter", 1);
    }

    for i in 0..10 {
        store.set_gauge("test.centroids", 554.0);
    }
    // for i in 0..10 {
    //     // Table: per-? metrics
    //     let unit_id = format!("user{}", i % 3); // Simulate 3 units
    //     let row_data = HashMap::from([
    //         (
    //             "reqs".to_string(),
    //             MetricValue::Counter((i % 10 + 1) as u64),
    //         ),
    //         (
    //             "lat".to_string(),
    //             MetricValue::Gauge((i % 5 + 1) as f64),
    //         ),
    //         (
    //             "lon".to_string(),
    //             MetricValue::Gauge((i % 5 + 1) as f64),
    //         ),

    //     ]);    
    //     store.add_table_row("units", &unit_id, row_data);        
    // }




    let metrics = store.get_all();
    let counter = store.get("test.counter");
    assert!(counter.is_some());
    let counter: Metric = counter.unwrap();
    assert_eq!(counter.key, "test.counter");
    assert!(matches!(counter.value, MetricValue::Counter(_)));
    println!("Counter value: {:?}", counter.value);


    let metrics = store.get_all();
    let metrics = serde_json::to_string_pretty(&metrics).unwrap();
    println!("{}", metrics);


//     for metric in metrics {
//         let metric = serde_json::to_string(&metric).unwrap();
//         println!("{}", metric);
//     }
// //     assert_eq!(counter, 10);

}