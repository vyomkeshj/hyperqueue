use std::time::SystemTime;
use termion::event::Key;

use crate::dashboard::ui::screen::Screen;
use crate::dashboard::ui::styles::{style_footer, style_header_text};
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::dashboard::ui::widgets::text::draw_text;

use crate::dashboard::data::alloc_timeline::AllocationQueueInfo;
use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::screen::controller::ScreenController;
use crate::dashboard::ui::screens::auto_allocator::queue_info_table::AllocationQueueInfoTable;
use crate::server::autoalloc::DescriptorId;
use tui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Default)]
pub struct AutoAllocatorScreen {
    queue_info_table: AllocationQueueInfoTable,
}

impl Screen for AutoAllocatorScreen {
    fn draw(&mut self, frame: &mut DashboardFrame) {
        let layout = AutoAllocScreenLayout::new(frame);
        draw_text(
            "AutoAlloc Info",
            layout.header_chunk,
            frame,
            style_header_text(),
        );
        self.queue_info_table.draw(layout.queue_info_chunk, frame);

        draw_text(
            "Press right_arrow to go to Cluster Overview",
            layout.footer_chunk,
            frame,
            style_footer(),
        );
    }

    fn update(&mut self, data: &DashboardData, _controller: &mut ScreenController) {
        let queue_infos: Vec<(&DescriptorId, &AllocationQueueInfo)> =
            data.query_allocation_queues_at(SystemTime::now()).collect();
        self.queue_info_table.update(queue_infos);
    }

    /// Handles key presses for the components of the screen
    fn handle_key(&mut self, key: Key, controller: &mut ScreenController) {
        match key {
            Key::Down => (),
            Key::Up => (),
            Key::Right => controller.show_cluster_overview(),
            _ => {}
        }
    }
}

/**
*  __________________________
   |     Queue Params      |
   |--------Header---------|
   |-----------------------|
   |  q_info  | alloc_info |
   -------------------------
   |________Footer_________|
 **/
struct AutoAllocScreenLayout {
    _allocation_queue_params_chunk: Rect,
    header_chunk: Rect,
    queue_info_chunk: Rect,
    _allocation_info_chunk: Rect,
    footer_chunk: Rect,
}

impl AutoAllocScreenLayout {
    fn new(frame: &DashboardFrame) -> Self {
        let auto_alloc_screen_chunks = tui::layout::Layout::default()
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(5),
                Constraint::Percentage(60),
                Constraint::Percentage(5),
            ])
            .direction(Direction::Vertical)
            .split(frame.size());

        let table_chunks = Layout::default()
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Horizontal)
            .margin(0)
            .split(auto_alloc_screen_chunks[2]);

        Self {
            _allocation_queue_params_chunk: auto_alloc_screen_chunks[0],
            header_chunk: auto_alloc_screen_chunks[1],
            queue_info_chunk: table_chunks[0],
            _allocation_info_chunk: table_chunks[1],
            footer_chunk: auto_alloc_screen_chunks[3],
        }
    }
}
