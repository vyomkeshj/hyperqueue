use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::ui::widgets::chart::{ChartDatapoint, DataFetcher, UtilChart};
use std::time::{SystemTime, UNIX_EPOCH};
use tako::WorkerId;
use tui::layout::Rect;
use tui::style::Color;

#[derive(Default)]
pub struct JobHwUtilChart {
    /// The worker the selected task is running on.
    worker_id: Option<WorkerId>,
    /// The chart.
    chart: UtilChart,
}

struct CpuUtilDataFetcher {
    /// The worker id for the chart data.
    worker_id: Option<WorkerId>,
}

impl JobHwUtilChart {
    pub fn set_worker_id(&mut self, worker_id: WorkerId) {
        self.worker_id = Some(worker_id);
    }
}

impl JobHwUtilChart {
    pub fn update(&mut self, data: &DashboardData) {
        let cpu_data_fetcher: Box<dyn DataFetcher> = Box::new(CpuUtilDataFetcher {
            worker_id: self.worker_id,
        });

        self.chart.add_data_fetcher(cpu_data_fetcher);
        self.chart.update(data);
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        self.chart.draw(rect, frame);
    }
}

impl DataFetcher for CpuUtilDataFetcher {
    /// Fetch the average CPU utilization for the worker.
    fn fetch_datapoint(&self, data: &DashboardData, at_time: SystemTime) -> Option<ChartDatapoint> {
        if let Some(hw_state) = self
            .worker_id
            .and_then(|worker_id| data.query_worker_overview_at(worker_id, at_time))
            .and_then(|overview| overview.hw_state.as_ref())
        {
            let per_core_util_vec = &hw_state.state.worker_cpu_usage.cpu_per_core_percent_usage;
            if per_core_util_vec.is_empty() {
                return None;
            }
            let average_cpu_util =
                per_core_util_vec.iter().sum::<f32>() as f64 / per_core_util_vec.len() as f64;

            return Some(ChartDatapoint {
                key: self.worker_id.unwrap().to_string(),
                data: (get_time_as_secs(at_time), average_cpu_util),
                color: Color::Red,
            });
        }
        None
    }
}

fn get_time_as_secs(time: SystemTime) -> f64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs() as f64
}
