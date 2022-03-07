use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tako::common::Map;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::symbols;
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset};

use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::styles::chart_style_deselected;
use crate::dashboard::ui::terminal::DashboardFrame;

/// Chart used to show cpu and memory usage across time.
pub struct UtilChart {
    /// Different data fetchers for the data shown on the chart.
    data_fetchers: Vec<Box<dyn DataFetcher>>,
    /// Chart's data, for each of the different plots on the chart.  
    datasets: Map<String, Vec<(f64, f64)>>,
    /// The end time of the data displayed on the chart,
    end_time: SystemTime,
    /// The duration for which the data is plotted for.
    view_size: Duration,
}

/// Fetches the data for a graph on the chart.
pub trait DataFetcher {
    fn fetch_datapoint(&self, data: &DashboardData, at_time: SystemTime) -> Option<ChartDatapoint>;
}

/// One of the items plotted on the chart.
pub struct ChartDatapoint {
    /// Each `chart_datapoint` that is plotted has its key.
    pub key: String,
    /// (x, y) coordinate of a point on the chart.
    pub data: (f64, f64),
    /// The color representing the data on the chart.
    pub color: Color,
}

impl UtilChart {
    pub fn add_data_fetcher(&mut self, data_fetcher: Box<dyn DataFetcher>) {
        self.data_fetchers.push(data_fetcher);
    }

    pub fn update(&mut self, data: &DashboardData) {
        self.end_time = SystemTime::now();
        self.datasets.clear();

        let mut start = self.end_time - self.view_size;
        let mut times = vec![];

        while start <= self.end_time {
            times.push(start);
            start += Duration::from_secs(1);
        }

        times
            .into_iter()
            .flat_map(|time| {
                // One data_point for each of the data fetchers. (String, (f64, f64))
                self.data_fetchers
                    .iter()
                    .filter_map(move |fetcher| fetcher.fetch_datapoint(data, time))
                    .map(|data_point| (data_point.key, data_point.data))
            })
            .for_each(|(key, point)| {
                if let Some(points) = self.datasets.get_mut(&*key) {
                    points.push(point);
                } else {
                    self.datasets.insert(key, vec![point]);
                }
            });
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        let datasets: Vec<Dataset> = self
            .datasets
            .iter()
            .map(|(label, dataset)| {
                Dataset::default()
                    .name(label)
                    .marker(symbols::Marker::Braille)
                    .style(Style::default())
                    .data(dataset)
            })
            .collect();

        let chart = Chart::new(datasets)
            .style(chart_style_deselected())
            .block(
                Block::default()
                    .title(Span::styled(
                        "Worker Hardware Utilization",
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD),
                    ))
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("time ->")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([
                        get_time_as_secs(self.end_time - self.view_size) as f64,
                        get_time_as_secs(self.end_time) as f64,
                    ]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.00, 100.00]),
            );
        //todo: add labels
        frame.render_widget(chart, rect);
    }
}

impl Default for UtilChart {
    fn default() -> Self {
        Self {
            data_fetchers: vec![],
            datasets: Default::default(),
            end_time: SystemTime::now(),
            view_size: Duration::from_secs(300),
        }
    }
}

fn get_time_as_secs(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}
