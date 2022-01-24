use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::ui::widgets::progressbar::{
    get_progress_bar_color, render_progress_bar_at, ProgressPrintStyle,
};
use crate::dashboard::ui::widgets::table::{StatefulTable, TableColumnHeaders};
use crate::dashboard::utils::{calculate_memory_usage_percent, get_average_cpu_usage_for_worker};
use tako::messages::worker::WorkerOverview;
use tako::WorkerId;
use tui::layout::{Constraint, Rect};
use tui::widgets::{Cell, Row};

#[derive(Default)]
pub struct PerCoreUtilTable {
    table: StatefulTable<WorkerCpuBarRow>,
}

impl PerCoreUtilTable {
    pub fn update(&mut self, data: &DashboardData) {
        let overview = data.query_latest_overview();
        let rows = create_rows(overview);
        self.table.set_items(rows);
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        // todo: decide the number of columns based on dashboard frame's size
        // todo: then partially fill them one by one with progress bars
        self.table.draw(
            rect,
            frame,
            TableColumnHeaders {
                title: "cpu usage per core".to_string(),
                inline_help: "".to_string(),
                table_headers: None,
                column_widths: vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ],
            },
            |data| {
                let cpu_progress = (data.average_cpu_usage.unwrap_or(0.0)) / 100.0;

                //todo: iterate to get cpu usage rows
                Row::new(vec![
                    Cell::from(data.id.to_string()),
                    Cell::from(data.num_tasks.to_string()),
                    Cell::from(cpu_prog_bar).style(get_progress_bar_color(cpu_progress)),
                    Cell::from(mem_prog_bar).style(get_progress_bar_color(mem_progress)),
                ])
            },
        );
    }
}

struct WorkerCpuBarRow {
    id: WorkerId,
    average_cpu_usage: Option<f32>,
}

fn create_rows(overview: Vec<&WorkerOverview>) -> Vec<WorkerCpuBarRow> {
    overview
        .iter()
        .map(|worker| {
            let hw_state = worker.hw_state.as_ref();
            let average_cpu_usage = hw_state.map(get_average_cpu_usage_for_worker);
            WorkerCpuBarRow {
                id: worker.id,
                average_cpu_usage
            }
        })
        .collect()
}
