use termion::event::Key;

use crate::dashboard::ui::screen::Screen;
use crate::dashboard::ui::styles::style_header_text;
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::ui::widgets::text::draw_text;

use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::screens::worker::worker_info_table::WorkerInfoTable;
use tui::layout::{Constraint, Direction, Layout, Rect};
use crate::dashboard::ui::screens::worker::worker_cores_util::PerCoreUtilTable;
use crate::dashboard::ui::screens::worker::worker_tasks_table::WorkerTasksTable;

#[derive(Default)]
pub struct WorkerScreen {
    tasks_table: WorkerTasksTable,
    worker_info_table: WorkerInfoTable,
    worker_cpu_bars: PerCoreUtilTable,
    //todo: what about worker job history table?
}

impl Screen for WorkerScreen {
    fn draw(&mut self, frame: &mut DashboardFrame) {
        let layout = WorkerInfoLayout::new(frame);
        //todo: insert worker_id
        draw_text("Worker info for: ", layout.header_chunk, frame, style_header_text());

        self.worker_cpu_bars.draw(layout.chart_chunk, frame);
        self.tasks_table.draw(layout.body_chunk, frame);
        self.worker_info_table.draw(layout.info_chunk, frame);
    }

    //todo: should have the current worker id selected
    fn update(&mut self, data: &DashboardData) {
        self.tasks_table.update(data);
        self.worker_cpu_bars.update(data);
        self.worker_info_table
            .update(data, self.tasks_table.get_selected_item());
    }

    /// Handles key presses for the components of the screen
    fn handle_key(&mut self, key: Key) {
        match key {
            Key::Down => self.tasks_table.select_next_task(),
            Key::Up => self.tasks_table.select_previous_task(),
            _ => {}
        }
    }
}

/**
*  __________________________
   |     Chart |    Info   |
   |--------Header---------|
   |-----------------------|
   |          BODY         |
   -------------------------
 **/
struct WorkerInfoLayout {
    chart_chunk: Rect,
    info_chunk: Rect,
    header_chunk: Rect,
    body_chunk: Rect,
}

impl WorkerInfoLayout {
    fn new(frame: &DashboardFrame) -> Self {
        let base_chunks = tui::layout::Layout::default()
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(10),
                Constraint::Percentage(30),
            ])
            .direction(Direction::Vertical)
            .split(frame.size());

        let info_chunks = Layout::default()
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(Direction::Horizontal)
            .margin(0)
            .split(base_chunks[0]);

        Self {
            chart_chunk: info_chunks[0],
            info_chunk: info_chunks[1],
            header_chunk: base_chunks[1],
            body_chunk: base_chunks[2],
        }
    }
}

pub fn vertical_chunks(constraints: Vec<Constraint>, size: Rect) -> Vec<Rect> {
    tui::layout::Layout::default()
        .constraints(constraints.as_ref())
        .direction(Direction::Vertical)
        .split(size)
}

pub fn horizontal_chunks_with_margin(
    constraints: Vec<Constraint>,
    size: Rect,
    margin: u16,
) -> Vec<Rect> {
    Layout::default()
        .constraints(constraints.as_ref())
        .direction(Direction::Horizontal)
        .margin(margin)
        .split(size)
}
