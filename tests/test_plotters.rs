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

/// Plot metrics using Plotters
fn plot_metrics(store: Arc<MetricStore>) {

    loop {
        let metrics = store.get_all();

        // Setup Plotters
        let root = BitMapBackend::new("telemetry.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();


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
        for (i, history) in metrics.iter().enumerate() {
            let label = format!("{:?}", history.key);
            chart
                .draw_series(LineSeries::new(
                    history.value.iter().map(|&(t, v)| (t as f64, v)),
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

        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .background_style(&WHITE.mix(0.8))
            .draw()
            .unwrap();

        root.present().unwrap();
    }
}