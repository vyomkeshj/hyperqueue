use crate::server::event::events::{JobInfo, MonitoringEventPayload};
use crate::server::event::MonitoringEvent;
use crate::{JobId, TakoTaskId, WorkerId};
use chrono::{DateTime, Utc};
use std::time::SystemTime;
use tako::common::Map;

pub struct DashboardJobInfo {
    pub job_info: JobInfo,
    pub job_tasks_info: Map<TakoTaskId, TaskInfo>,
    pub completion_date: Option<DateTime<Utc>>,
}

pub struct TaskInfo {
    pub worker_id: WorkerId,
    pub start_time: SystemTime,
    pub end_time: Option<SystemTime>,
    pub current_task_state: DashboardTaskState,
}

#[derive(Copy, Clone)]
pub enum DashboardTaskState {
    Running,
    Finished,
    Failed,
}

impl TaskInfo {
    pub fn set_end_time_and_status(&mut self, end_time: &SystemTime, status: DashboardTaskState) {
        self.end_time = Some(*end_time);
        self.current_task_state = status;
    }
}

#[derive(Default)]
pub struct JobTimeline {
    job_timeline: Map<JobId, DashboardJobInfo>,
}

impl JobTimeline {
    /// Assumes that `events` are sorted by time.
    pub fn handle_new_events(&mut self, events: &[MonitoringEvent]) {
        for event in events {
            match &event.payload {
                MonitoringEventPayload::JobCreated(job_id, job_info) => {
                    self.job_timeline.insert(
                        *job_id,
                        DashboardJobInfo {
                            job_info: *job_info.clone(),
                            job_tasks_info: Default::default(),
                            completion_date: None,
                        },
                    );
                }

                MonitoringEventPayload::JobCompleted(job_id, completion_date) => {
                    if let Some(job_info) = self.job_timeline.get_mut(job_id) {
                        job_info.completion_date = Some(*completion_date)
                    }
                }

                MonitoringEventPayload::TaskStarted { task_id, worker_id } => {
                    self.job_timeline
                        .iter_mut()
                        .find(|(_, info)| info.job_info.task_ids.contains(task_id))
                        .and_then(|(_, info)| {
                            info.job_tasks_info.insert(
                                *task_id,
                                TaskInfo {
                                    worker_id: *worker_id,
                                    start_time: event.time,
                                    end_time: None,
                                    current_task_state: DashboardTaskState::Running,
                                },
                            )
                        });
                }
                MonitoringEventPayload::TaskFinished(finished_id) => {
                    update_task_status(
                        &mut self.job_timeline,
                        finished_id,
                        DashboardTaskState::Finished,
                        &event.time,
                    );
                }
                MonitoringEventPayload::TaskFailed(failed_id) => {
                    update_task_status(
                        &mut self.job_timeline,
                        failed_id,
                        DashboardTaskState::Failed,
                        &event.time,
                    );
                }
                _ => {}
            }
        }
    }

    pub fn get_worker_task_history(
        &self,
        worker_id: WorkerId,
        at_time: SystemTime,
    ) -> impl Iterator<Item = (&TakoTaskId, &TaskInfo)> + '_ {
        self.job_timeline
            .iter()
            .flat_map(|(_, info)| &info.job_tasks_info)
            .filter(move |(_, info)| info.worker_id == worker_id && info.start_time <= at_time)
    }
}

fn update_task_status(
    job_timeline: &mut Map<JobId, DashboardJobInfo>,
    task_id: &TakoTaskId,
    task_status: DashboardTaskState,
    at_time: &SystemTime,
) {
    if let Some((_, job_info)) = job_timeline
        .iter_mut()
        .find(|(_, info)| info.job_info.task_ids.contains(task_id))
    {
        if let Some(task_info) = job_info.job_tasks_info.get_mut(task_id) {
            task_info.set_end_time_and_status(at_time, task_status);
        }
    };
}
