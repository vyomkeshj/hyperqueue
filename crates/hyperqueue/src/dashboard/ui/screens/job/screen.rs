use std::default::Default;
use std::time::SystemTime;
use termion::event::Key;

use crate::dashboard::ui::screen::Screen;
use crate::dashboard::ui::styles::{
    style_footer, style_header_text, table_style_deselected, table_style_selected,
};
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::ui::widgets::text::draw_text;

use crate::dashboard::data::job_timeline::TaskInfo;
use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::screen::controller::ScreenController;

use crate::dashboard::ui::screens::job::job_info_table::JobsTable;
use crate::dashboard::ui::widgets::tasks_table::TasksTable;

use crate::TakoTaskId;
use tui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Default)]
pub struct JobScreen {
    job_info_table: JobsTable,
    job_tasks_table: TasksTable,

    component_in_focus: FocusedComponent,
}

enum FocusedComponent {
    JobInfoTable,
    JobTasksTable,
}

impl Screen for JobScreen {
    fn draw(&mut self, frame: &mut DashboardFrame) {
        let layout = JobScreenLayout::new(frame);
        draw_text("Job Info", layout.header_chunk, frame, style_header_text());

        let (jobs_table_style, tasks_table_style) = match self.component_in_focus {
            FocusedComponent::JobInfoTable => (table_style_selected(), table_style_deselected()),
            FocusedComponent::JobTasksTable => (table_style_deselected(), table_style_selected()),
        };

        self.job_info_table
            .draw(layout.job_info_chunk, frame, tasks_table_style);
        self.job_tasks_table.draw(
            "Started Tasks <2>",
            layout.job_tasks_chunk,
            frame,
            jobs_table_style,
        );

        draw_text(
            "Press -> to go to Allocations Screen",
            layout.footer_chunk,
            frame,
            style_footer(),
        );
    }

    fn update(&mut self, data: &DashboardData, _controller: &mut ScreenController) {
        self.job_info_table.update(data);

        if let Some(job_id) = self.job_info_table.get_selected_item() {
            let task_infos: Vec<(&TakoTaskId, &TaskInfo)> = data
                .query_task_history_for_job(job_id, SystemTime::now())
                .collect();
            self.job_tasks_table.update(task_infos);
        }
    }

    /// Handles key presses for the components of the screen
    fn handle_key(&mut self, key: Key, controller: &mut ScreenController) {
        match self.component_in_focus {
            FocusedComponent::JobInfoTable => self.job_info_table.handle_key(key),
            FocusedComponent::JobTasksTable => self.job_tasks_table.handle_key(key),
        };

        match key {
            Key::Right => controller.show_auto_allocator_screen(),
            Key::Char('1') => {
                self.component_in_focus = FocusedComponent::JobInfoTable;
                self.job_tasks_table.clear_selection();
            }
            Key::Char('2') => self.component_in_focus = FocusedComponent::JobTasksTable,
            _ => {}
        }
    }
}

/**
*  _________________________
   |--------Header---------|
   |        Chart          |
   |-----------------------|
   |  j_info  |   j_tasks  |
   |________Footer_________|
 **/
struct JobScreenLayout {
    header_chunk: Rect,
    _chart_chunk: Rect,
    job_info_chunk: Rect,
    job_tasks_chunk: Rect,
    footer_chunk: Rect,
}

impl JobScreenLayout {
    fn new(frame: &DashboardFrame) -> Self {
        let job_screen_chunks = tui::layout::Layout::default()
            .constraints(vec![
                Constraint::Percentage(5),
                Constraint::Percentage(40),
                Constraint::Percentage(50),
                Constraint::Percentage(5),
            ])
            .direction(Direction::Vertical)
            .split(frame.size());

        let component_area = Layout::default()
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .direction(Direction::Horizontal)
            .margin(0)
            .split(job_screen_chunks[2]);

        Self {
            header_chunk: job_screen_chunks[0],
            _chart_chunk: job_screen_chunks[1],
            job_info_chunk: component_area[0],
            job_tasks_chunk: component_area[1],
            footer_chunk: job_screen_chunks[3],
        }
    }
}

impl Default for FocusedComponent {
    fn default() -> Self {
        FocusedComponent::JobInfoTable
    }
}
