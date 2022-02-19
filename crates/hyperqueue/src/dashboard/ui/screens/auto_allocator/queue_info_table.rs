use crate::dashboard::data::alloc_timeline::AllocationQueueInfo;
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::ui::widgets::table::{StatefulTable, TableColumnHeaders};
use crate::server::autoalloc::DescriptorId;
use chrono::{DateTime, Local};
use tui::layout::{Constraint, Rect};
use tui::widgets::{Cell, Row};

#[derive(Default)]
pub struct AllocationQueueInfoTable {
    table: StatefulTable<QueueInfoRow>,
}

impl AllocationQueueInfoTable {
    pub fn update(&mut self, queue_infos: Vec<(&DescriptorId, &AllocationQueueInfo)>) {
        let rows = create_rows(queue_infos);
        self.table.set_items(rows);
    }

    pub fn select_next_queue(&mut self) {
        self.table.select_next_wrap();
    }

    pub fn select_previous_queue(&mut self) {
        self.table.select_previous_wrap();
    }

    pub fn get_selected_queue_descriptor(&self) -> Option<DescriptorId> {
        let selection = self.table.current_selection();
        selection.map(|row| row.descriptor_id)
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame, is_selected: bool) {
        self.table.draw(
            rect,
            frame,
            TableColumnHeaders {
                title: "Allocation Queues <1>",
                inline_help: "",
                table_headers: Some(vec![
                    "Descriptor ID",
                    "#Allocations",
                    "Creation Time",
                    "Removal Time",
                ]),
                column_widths: vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ],
            },
            |data| {
                Row::new(vec![
                    Cell::from(data.descriptor_id.to_string()),
                    Cell::from(data.num_allocations.to_string()),
                    Cell::from(data.creation_time.as_str()),
                    Cell::from(data.removal_time.as_str()),
                ])
            },
            is_selected,
        );
    }
}

struct QueueInfoRow {
    descriptor_id: DescriptorId,
    num_allocations: u32,
    creation_time: String,
    removal_time: String,
}

fn create_rows(queues: Vec<(&DescriptorId, &AllocationQueueInfo)>) -> Vec<QueueInfoRow> {
    queues
        .iter()
        .map(|(descriptor_id, info)| {
            let creation_time: DateTime<Local> = info.creation_time.into();
            let removal_time = info
                .removal_time
                .map(|time| {
                    let end_time: DateTime<Local> = time.into();
                    end_time.format("%b %e, %T").to_string()
                })
                .unwrap_or_else(|| "".to_string());

            QueueInfoRow {
                descriptor_id: **descriptor_id,
                num_allocations: info.allocations.len() as u32,
                creation_time: creation_time.format("%b %e, %T").to_string(),
                removal_time,
            }
        })
        .collect()
}
