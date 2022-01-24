use crate::WorkerId;
use std::time::SystemTime;
use tako::messages::common::WorkerConfiguration;
use tako::messages::gateway::{LostWorkerReason, TaskState};
use tako::server::monitoring::{MonitoringEvent, MonitoringEventPayload};
use tako::TaskId;

#[derive(Clone)]
pub struct TaskMessage {
    worker_id: WorkerId,
    task_id: TaskId,
    task_state: TaskState,
}

/// Stores information about the workers at different times
#[derive(Default)]
pub struct TasksTimeline {
    tasks_timeline: Vec<TaskMessage>,
}

impl TasksTimeline {
    /// Assumes that `events` are sorted by time.
    pub fn handle_new_events(&mut self, events: &[MonitoringEvent]) {
        for event in events {
            match &event.payload {
                MonitoringEventPayload::TaskRunning(worker_id, msg) => {

                },
                MonitoringEventPayload::TaskFinished(worker_id, msg) => {

                },
                MonitoringEventPayload::TaskFailed(worker_id, msg) => {

                }
                _ => {}
            }
        }
    }

    pub fn get_tasks_running_on_worker(&self, worker_id: &WorkerId) -> Option<&WorkerConfiguration> {
        return self
            .tasks_timeline
            .iter()
            .find(|info| info.worker_id == *worker_id)
            .map(|info| &info.worker_info);
    }

}
