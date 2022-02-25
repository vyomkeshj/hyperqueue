use std::time::{Duration, SystemTime};

use tako::common::Map;
use tui::layout::Rect;
use tui::widgets::canvas::{Canvas, Painter, Shape};

use crate::dashboard::data::alloc_timeline::AllocationInfo;
use crate::dashboard::data::DashboardData;
use crate::dashboard::ui::terminal::DashboardFrame;
use crate::server::autoalloc::{AllocationId, DescriptorId};
use chrono::{DateTime, Local};
use std::default::Default;
use tui::style::Color;
use tui::widgets::{Block, Borders};

const LABEL_Y_MARGIN: f64 = 1.00;
const END_LABEL_OFFSET: f64 = 20.00;

/// Stores the allocation state at a point in time.
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
    /// max time that is being shown currently
    max_time: SystemTime,
    /// The size of the viewing window
    view_size: Duration,
}

#[derive(Default)]
pub struct AllocationsChart {
    chart_data: AllocationsChartData,
}

impl Shape for AllocationsChartData {
    fn draw(&self, painter: &mut Painter) {
        let mut y_pos: f64 = LABEL_Y_MARGIN;
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
            y_pos += 1.0;
        }
    }
}

impl AllocationsChart {
    pub fn update(&mut self, data: &DashboardData, query_descriptor: DescriptorId) {
        self.chart_data.allocation_records.clear();
        self.chart_data.max_time = SystemTime::now();

        let mut query_time = self.chart_data.max_time - self.chart_data.view_size;
        let mut all_time_allocs: Vec<(AllocationId, AllocationInfoPoint)> = vec![];

        while query_time <= self.chart_data.max_time {
            query_time += Duration::from_secs(1);
            if let Some(alloc_map) = data.query_allocations_info_at(query_descriptor, query_time) {
                let mut points: Vec<(AllocationId, AllocationInfoPoint)> = alloc_map
                    .iter()
                    .map(|(alloc_id, alloc_info)| {
                        (
                            alloc_id.clone(),
                            AllocationInfoPoint {
                                current_state: get_alloc_state(alloc_info, query_time),
                                time: query_time,
                            },
                        )
                    })
                    .collect();
                all_time_allocs.append(&mut points);
            }
        }

        for (id, point) in all_time_allocs {
            if let Some(points) = self.chart_data.allocation_records.get_mut(&id) {
                points.push(point);
            } else {
                self.chart_data.allocation_records.insert(id, vec![point]);
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
                let current_max: DateTime<Local> = self.chart_data.max_time.into();
                let current_min: DateTime<Local> =
                    (self.chart_data.max_time - self.chart_data.view_size).into();

                ctx.print(0.0, 0.0, current_min.format("%T").to_string());
                ctx.print(
                    self.chart_data.view_size.as_secs_f64() - END_LABEL_OFFSET,
                    0.0,
                    current_max.format("%T").to_string(),
                );
            })
            .x_bounds([0.0, self.chart_data.view_size.as_secs_f64()])
            .y_bounds([
                0.0,
                self.chart_data.allocation_records.len() as f64 + LABEL_Y_MARGIN,
            ]);
        frame.render_widget(canvas, rect);
    }
}

fn get_alloc_state(info: &AllocationInfo, query_time: SystemTime) -> AllocState {
    let has_started = info.start_time.is_some() && info.start_time.unwrap() < query_time;
    let has_finished = match info.finish_time {
        None => false,
        Some(finish_time) => finish_time < query_time,
    };
    match has_started && !has_finished {
        true => (AllocState::Running),
        false => match has_finished {
            true => AllocState::Finished,
            false => AllocState::Queued,
        },
    }
}

impl Default for AllocationsChartData {
    fn default() -> Self {
        Self {
            view_size: Duration::from_secs(600),
            allocation_records: Default::default(),
            max_time: SystemTime::now(),
        }
    }
}
