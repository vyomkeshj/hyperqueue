use std::time::{SystemTime, UNIX_EPOCH};
use tako::messages::gateway::CollectedOverview;

use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::utils::get_average_cpu_usage_for_worker;
use std::collections::HashMap;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::symbols;
use tui::text::Span;
use tui::widgets::{Axis, BarChart, Block, Borders, Chart, Dataset};

#[derive(Clone)]
pub struct WorkerCpuUtilChart {
    /// The state of the worker at different times.
    cpu_core_data: HashMap<u32, Vec<f32>>,
    current_worker_id: u32,
}

impl WorkerCpuUtilChart {
    pub fn update(&mut self, overview: CollectedOverview) {
        // Update the cpu usage data
        self.cpu_core_data.clear();
        let mut avg_util: Vec<f32> = vec![];
        for worker_ov in overview.clone().worker_overviews {
            if let Some(hw_overview) = worker_ov.hw_state {
                avg_util = hw_overview
                    .state
                    .worker_cpu_usage
                    .cpu_per_core_percent_usage;
                self.cpu_core_data.insert(worker_ov.id, avg_util);
            }
        }
    }

    pub fn update_selected_worker(&mut self, new_worker_id: Option<u32>) {
        if let Some(new_worker_id) = new_worker_id {
            self.current_worker_id = new_worker_id;
        }
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        if let Some(cpu_util) = self.cpu_core_data.get(&self.current_worker_id).clone() {
            let dataset: Vec<(&str, u64)> = cpu_util.iter().map(|x| ("___", *x as u64)).collect();
            let chart = BarChart::default()
                .block(
                    Block::default()
                        .title("Worker's CPU usage per core")
                        .borders(Borders::ALL),
                )
                .bar_width(4)
                .bar_gap(1)
                .style(Style::default().fg(Color::Yellow).bg(Color::Black))
                .data(&dataset)
                .max(100);
            frame.render_widget(chart, rect);
        }
    }
}

impl Default for WorkerCpuUtilChart {
    fn default() -> Self {
        Self {
            cpu_core_data: HashMap::new(),
            current_worker_id: 0,
        }
    }
}
