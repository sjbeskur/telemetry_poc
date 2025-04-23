use telemetry_poc::*;


#[test]
fn test_counter() {

    let store = MetricStore::new();
    for i in 0..10 {
        store.increment_counter("test.counter", 1);
    }

    let metrics = store.get_all();
    let counter = store.get("test.counter");
    assert!(counter.is_some());
    let counter: Metric = counter.unwrap();
    assert_eq!(counter.name, "test.counter");
    assert!(matches!(counter.value, MetricValue::Counter(_)));
    println!("Counter value: {:?}", counter.value);

    for metric in metrics {
        println!("Counter value: {:?}", metric);
    }
//     assert_eq!(counter, 10);

}