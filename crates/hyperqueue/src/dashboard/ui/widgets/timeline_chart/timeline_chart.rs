use std::time::{Duration, SystemTime};

use tako::common::Map;
use tui::layout::Rect;
use tui::widgets::canvas::{Canvas, Painter, Shape};

use crate::dashboard::data::DashboardData;
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

struct AllocationsChartData {
    /// Allocation chart data.
    allocation_records: Map<AllocationId, Vec<AllocationInfoPoint>>,
    /// max time to calculate duration
    max_time: SystemTime,
    /// The size of the viewing window
    view_size: Duration,
}

pub struct AllocationsChart {
    chart_data: AllocationsChartData,
}

impl Shape for AllocationsChartData {
    fn draw(&self, painter: &mut Painter) {
        let mut y_pos: f64 = 0.0;
        for (_, alloc_info_pt) in &self.allocation_records {
            for point in alloc_info_pt {
                let x_pos = self.view_size.as_secs_f64()
                    - self
                        .max_time
                        .duration_since(point.time)
                        .unwrap_or_default()
                        .as_secs_f64();
                if let Some((x, y)) = painter.get_point(x_pos, y_pos) {
                    painter.paint(
                        x,
                        y,
                        match point.current_state {
                            AllocState::Queued => Color::Yellow,
                            AllocState::Running => Color::Green,
                            AllocState::Finished => Color::Red,
                        },
                    );
                }
            }
            y_pos += 1.0; // (ALLOC_HEIGHT / 2.0) + ALLOC_Y_MARGIN;
        }
    }
}

impl AllocationsChart {
    pub fn update(&mut self, data: &DashboardData, query_descriptor: DescriptorId) {
        self.chart_data.allocation_records.clear();
        self.chart_data.max_time = SystemTime::now();

        let mut query_time = self.chart_data.max_time - self.chart_data.view_size;
        while query_time <= self.chart_data.max_time {
            query_time += Duration::from_secs(1);

            if let Some(alloc_map) = data.query_allocations_info_at(query_descriptor, query_time) {
                let mut points: Vec<(AllocationId, AllocationInfoPoint)> = alloc_map
                    .iter()
                    .map(|(id, info)| {
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
                                time: query_time,
                            },
                        )
                    })
                    .collect();
                //fixme: check correctness
                for (id, point) in points {
                    if let Some(points) = self.chart_data.allocation_records.get_mut(&id) {
                        points.push(point);
                    } else {
                        self.chart_data.allocation_records.insert(id, vec![point]);
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, rect: Rect, frame: &mut DashboardFrame) {
        let canvas = Canvas::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Alloc Timeline"),
            )
            .paint(|ctx| {
                ctx.draw(&self.chart_data);
            })
            .x_bounds([0.0, self.chart_data.view_size.as_secs_f64()]) //fixme: use view size
            .y_bounds([0.0, self.chart_data.allocation_records.len() as f64]);
        frame.render_widget(canvas, rect);
    }
}

impl Default for AllocationsChartData {
    fn default() -> Self {
        Self {
            view_size: Duration::from_secs(300),
            allocation_records: Default::default(),
            max_time: SystemTime::now(),
        }
    }
}

impl Default for AllocationsChart {
    fn default() -> Self {
        Self {
            chart_data: Default::default(),
        }
    }
}
