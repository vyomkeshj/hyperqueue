use std::time::{Duration, SystemTime};

use tako::common::Map;
use tui::layout::Rect;
use tui::widgets::canvas::{Canvas, Painter, Shape};

use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::styles::style_deselected;
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::server::autoalloc::{AllocationId, DescriptorId};
use std::default::Default;
use tui::style::Color;
use tui::widgets::{Block, Borders};

const ALLOC_HEIGHT: f64 = 20.0;
const ALLOC_Y_MARGIN: f64 = 5.0;

struct AllocationInfoPoint {
    current_state: AllocState,
    time: SystemTime,
}

enum AllocState {
    Queued,
    Running,
    Finished,
}

pub struct AllocationsChartData {
    /// Allocation chart data.
    allocation_records: Map<AllocationId, AllocationInfoPoint>,
    /// max time to calculate duration
    max_time: SystemTime,
}

struct AllocationsChart {
    chart_data: AllocationsChartData,
    view_size: Duration,
}

impl Shape for AllocationsChartData {
    fn draw(&self, painter: &mut Painter) {
        let mut y_pos: f64 = 0.0;
        for (id, alloc_info_pt) in &self.allocation_records {
            //fixme: don't unwrap here
            let x_pos = self
                .max_time
                .duration_since(alloc_info_pt.time)
                .unwrap_or_default()
                .as_secs_f64();
            if let Some((x, y)) = painter.get_point(x_pos, y_pos) {
                painter.paint(
                    x,
                    y,
                    match alloc_info_pt.current_state {
                        AllocState::Queued => Color::Yellow,
                        AllocState::Running => Color::Green,
                        AllocState::Finished => Color::Red,
                    },
                );
            }
            y_pos += (ALLOC_HEIGHT / 2.0) + ALLOC_Y_MARGIN;
        }
    }
}

impl AllocationsChart {
    pub fn update(&mut self, data: &DashboardData, query_descriptor: DescriptorId) {
        let mut current_time = self.chart_data.max_time - self.view_size;
        let mut times = vec![];

        while current_time <= self.chart_data.max_time {
            times.push(current_time);
            current_time += Duration::from_secs(1);

            if let Some(alloc_map) = data.query_allocations_info_at(query_descriptor, current_time)
            {
                let points = alloc_map.iter().map(|(id, info)| {
                    let current_state = match info.start_time {
                        None => AllocState::Queued,
                        Some(_) => match info.finish_time {
                            None => AllocState::Running,
                            Some(_) => AllocState::Finished,
                        },
                    };
                    (
                        id.to_string(),
                        AllocationInfoPoint {
                            current_state,
                            time: current_time,
                        },
                    )
                });
                //fixme: check correctness
                self.chart_data.allocation_records.extend(points);
            }
        }
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        let canvas = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("World"))
            .paint(|ctx| {
                ctx.draw(&self.chart_data);
            })
            .x_bounds([0.0, self.view_size.as_secs_f64()])
            .y_bounds([0.0, self.chart_data.allocation_records.len() as f64]);
        frame.render_widget(canvas, rect);
    }
}

impl Default for AllocationsChartData {
    fn default() -> Self {
        Self {
            allocation_records: Default::default(),
            max_time: SystemTime::now(),
        }
    }
}

impl Default for AllocationsChart {
    fn default() -> Self {
        Self {
            view_size: Duration::from_secs(300),
            chart_data: Default::default(),
        }
    }
}
