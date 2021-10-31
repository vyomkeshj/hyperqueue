use std::time::{SystemTime, UNIX_EPOCH};
use tako::messages::gateway::CollectedOverview;

use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::utils::get_average_cpu_usage_for_worker;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::symbols;
use tui::text::Span;
use tui::widgets::{Axis, Block, Borders, Chart, Dataset};

/// A new WorkerTimeState is created everytime there is a #worker change event
#[derive(Clone)]
struct WorkerStateAtTime {
    /// The time at which the event being stored happened, wrt. the reference
    pub at_time: u128,
    pub event: TimeEventType,
}

#[derive(Clone)]
pub enum TimeEventType {
    /// Current number of workers.
    WorkerCountUpdate(u32),
    /// The average cpu utilization of the worker. percent, num_workers
    AvgCpuUsageUpdate(f32, f32),
}

#[derive(Clone)]
pub struct ClusterOverviewChart {
    /// The state of the worker at different times.
    timed_info_vec: Vec<WorkerStateAtTime>,
    /// Used to set the min bound of the chart's x-axis
    time_reference: u128,
    /// Used to set the max bound of the chart's y-axis
    max_observed_workers: u64,
    /// Current number of workers connected
    current_conn_workers: u64,
}

impl ClusterOverviewChart {
    pub fn update(&mut self, overview: CollectedOverview) {
        // Update the number of workers connected
        let time = get_current_time();
        let num_workers = overview.worker_overviews.len();
        if num_workers > self.max_observed_workers as usize {
            self.max_observed_workers = num_workers as u64;
        }
        self.timed_info_vec.push(WorkerStateAtTime {
            at_time: time,
            event: TimeEventType::WorkerCountUpdate(num_workers as u32),
        });
        // Add more points at the same time to smoothen the transition in plot
        let interpolation =
            interpolate_y_axis(self.current_conn_workers as f64, num_workers as f64, 60_f64);
        for y_point in interpolation {
            self.timed_info_vec.push(WorkerStateAtTime {
                at_time: time,
                event: TimeEventType::WorkerCountUpdate(y_point as u32),
            })
        }
        self.current_conn_workers = num_workers as u64;
        // Update the cpu usage data
        let mut avg_util: f32 = 0.0;
        for worker_ov in overview.clone().worker_overviews {
            if let Some(hw_overview) = worker_ov.hw_state {
                avg_util += get_average_cpu_usage_for_worker(&hw_overview.clone());
            }
        }
        avg_util = avg_util / overview.clone().worker_overviews.len() as f32;
        self.timed_info_vec.push(WorkerStateAtTime {
            at_time: time,
            event: TimeEventType::AvgCpuUsageUpdate((avg_util / 100.00), num_workers as f32),
        });

        let interpolation = interpolate_y_axis((avg_util / 100.00) as f64, 0 as f64, 60_f64);
        for y_point in interpolation {
            self.timed_info_vec.push(WorkerStateAtTime {
                at_time: time,
                event: TimeEventType::AvgCpuUsageUpdate(y_point as f32, num_workers as f32),
            })
        }
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        let worker_counts: Vec<(f64, f64)> = self.get_data_for_worker_count();

        let mut datasets = vec![];

        let cpu_data: Vec<(f64, f64, f64)> = self
            .timed_info_vec
            .iter()
            .filter_map(|event| match event.clone().event {
                TimeEventType::WorkerCountUpdate(count) => None,
                TimeEventType::AvgCpuUsageUpdate(util, num_workers) => {
                    Some((event.at_time as f64, util as f64, num_workers as f64))
                }
            })
            .collect();
        let high_usage: Vec<(f64, f64)> = cpu_data
            .clone()
            .into_iter()
            .filter(|(_, y, _)| (*y > 0.60))
            .map(|(time, mut data, scale)| (time, (data * scale)))
            .collect();
        let med_usage: Vec<(f64, f64)> = cpu_data
            .clone()
            .into_iter()
            .filter(|(_, y, _)| (*y >= 0.25 && *y < 0.60))
            .map(|(time, mut data, scale)| (time, (data * scale)))
            .collect();
        let low_usage: Vec<(f64, f64)> = cpu_data
            .clone()
            .into_iter()
            .filter(|(_, y, _)| (*y < 0.25))
            .map(|(time, mut data, scale)| (time, (data * scale)))
            .collect();

        datasets.push(
            Dataset::default()
                .name("high_usage")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Red))
                .data(&high_usage),
        );

        datasets.push(
            Dataset::default()
                .name("med_usage")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Magenta))
                .data(&med_usage),
        );

        datasets.push(
            Dataset::default()
                .name("low_usage")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::Blue))
                .data(&low_usage),
        );

        datasets.push(
            Dataset::default()
                .name("workers_connected")
                .marker(symbols::Marker::Braille)
                .style(Style::default().fg(Color::White))
                .data(&worker_counts),
        );

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title(Span::styled(
                        "Worker Connection History",
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
                    .bounds([self.time_reference as f64, get_current_time() as f64]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, self.max_observed_workers as f64])
                    .labels(vec![
                        Span::from(0.to_string()),
                        Span::from((self.max_observed_workers / 2).to_string()),
                        Span::from((self.max_observed_workers as f64).to_string()),
                    ]),
            );
        frame.render_widget(chart, rect);
    }

    fn get_data_for_worker_count(&self) -> Vec<(f64, f64)> {
        self.timed_info_vec
            .iter()
            .filter_map(|event| match event.clone().event {
                TimeEventType::WorkerCountUpdate(count) => {
                    Some((event.at_time as f64, count as f64))
                }
                TimeEventType::AvgCpuUsageUpdate(_, _) => None,
            })
            .collect()
    }
}

fn get_current_time() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_millis()
}

/**
 * Add extra points along the Y axis when the number of workers change to make the graph continuous
 */
fn interpolate_y_axis(y_a: f64, y_b: f64, density: f64) -> Vec<f64> {
    let mut y_max = if y_a > y_b { y_a } else { y_b };
    let y_min = if y_a < y_b { y_a } else { y_b };

    let decrement_interval = (y_max - y_min) / density as f64;
    let mut return_vec: Vec<f64> = vec![y_a as f64];
    while (y_max - y_min) > decrement_interval {
        y_max -= decrement_interval;
        return_vec.push(y_max as f64)
    }
    return_vec
}

impl Default for ClusterOverviewChart {
    fn default() -> Self {
        Self {
            timed_info_vec: vec![],
            time_reference: get_current_time(),
            max_observed_workers: 0,
            current_conn_workers: 0,
        }
    }
}
