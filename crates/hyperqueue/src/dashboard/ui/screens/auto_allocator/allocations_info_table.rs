use crate::dashboard::data::alloc_timeline::AllocationInfo;
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::ui::widgets::table::{StatefulTable, TableColumnHeaders};
use crate::server::autoalloc::AllocationId;
use chrono::{DateTime, Local};
use tako::common::Map;
use tui::layout::{Constraint, Rect};
use tui::widgets::{Cell, Row};

#[derive(Default)]
pub struct AllocationInfoTable {
    table: StatefulTable<AllocationInfoRow>,
}

impl AllocationInfoTable {
    pub fn update(&mut self, allocation_info: &Map<AllocationId, AllocationInfo>) {
        let rows = create_rows(allocation_info);
        self.table.set_items(rows);
    }

    pub fn select_next_allocation(&mut self) {
        self.table.select_next_wrap();
    }

    pub fn select_previous_allocation(&mut self) {
        self.table.select_previous_wrap();
    }

    pub fn get_selected_allocation_id(&self) -> Option<&AllocationId> {
        let selection = self.table.current_selection();
        selection.map(|row| &row.allocation_id)
    }

    pub fn clear_selection(&mut self) {
        self.table.clear_selection();
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        self.table.draw(
            rect,
            frame,
            TableColumnHeaders {
                title: "Allocations <2>".to_string(),
                inline_help: "".to_string(),
                table_headers: Some(vec![
                    "Allocation ID",
                    "#Workers",
                    "Queued Time",
                    "Start Time",
                    "Finish Time",
                ]),
                column_widths: vec![
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                ],
            },
            |data| {
                Row::new(vec![
                    Cell::from(data.allocation_id.as_str()),
                    Cell::from(data.num_workers.to_string()),
                    Cell::from(data.queued_time.as_str()),
                    Cell::from(data.start_time.as_str()),
                    Cell::from(data.finish_time.as_str()),
                ])
            },
        );
    }
}

struct AllocationInfoRow {
    allocation_id: AllocationId,
    num_workers: u64,
    queued_time: String,
    start_time: String,
    finish_time: String,
}

fn create_rows(allocations: &Map<AllocationId, AllocationInfo>) -> Vec<AllocationInfoRow> {
    allocations
        .iter()
        .map(|(alloc_id, info)| {
            let queued_time: DateTime<Local> = info.queued_time.into();
            let start_time = info
                .start_time
                .map(|time| {
                    let end_time: DateTime<Local> = time.into();
                    end_time.format("%b %e, %T").to_string()
                })
                .unwrap_or_else(|| "".to_string());
            let finish_time = info
                .finish_time
                .map(|time| {
                    let end_time: DateTime<Local> = time.into();
                    end_time.format("%b %e, %T").to_string()
                })
                .unwrap_or_else(|| "".to_string());

            AllocationInfoRow {
                allocation_id: alloc_id.to_string(),
                num_workers: info.worker_count,
                queued_time: queued_time.format("%b %e, %T").to_string(),
                start_time,
                finish_time,
            }
        })
        .collect()
}
