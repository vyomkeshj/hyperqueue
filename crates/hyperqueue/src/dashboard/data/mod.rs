use crate::dashboard::data::worker_timeline::WorkerTimeline;
use std::time::{Duration, SystemTime};
use tako::common::WrappedRcRefCell;
use tako::messages::common::WorkerConfiguration;
use tako::messages::gateway::MonitoringEventRequest;
use tako::messages::worker::WorkerOverview;

use crate::dashboard::data::alloc_timeline::{
    AllocationInfo, AllocationQueueInfo, AllocationTimeline,
};
use crate::dashboard::data::job_timeline::{JobTimeline, TaskInfo};
use crate::server::autoalloc::{AllocationId, DescriptorId};
use crate::server::event::MonitoringEvent;
use crate::transfer::connection::ClientConnection;
use crate::transfer::messages::{AllocationQueueParams, FromClientMessage, ToClientMessage};
use crate::{rpc_call, TakoTaskId, WorkerId};

pub mod alloc_timeline;
pub mod job_timeline;
pub mod worker_timeline;

#[derive(Default)]
pub struct DashboardData {
    /// The event_id until which the data has already been fetched for
    fetched_until: Option<u32>,
    /// Tracks worker connection and loss events
    worker_timeline: WorkerTimeline,
    /// Tracks task related events
    tasks_timeline: JobTimeline,
    /// Tracks the automatic allocator events
    alloc_timeline: AllocationTimeline,
}

impl DashboardData {
    pub fn last_fetched_id(&self) -> Option<u32> {
        self.fetched_until
    }

    pub fn update_data(&mut self, mut events: Vec<MonitoringEvent>) {
        events.sort_unstable_by_key(|e| e.time());

        // Update maximum event ID
        self.fetched_until = events
            .iter()
            .map(|event| event.id())
            .max()
            .or(self.fetched_until);

        // Update data views
        self.worker_timeline.handle_new_events(&events);
        self.tasks_timeline.handle_new_events(&events);
        self.alloc_timeline.handle_new_events(&events);
    }

    pub fn query_task_history_for_worker(
        &self,
        worker_id: WorkerId,
    ) -> impl Iterator<Item = (&TakoTaskId, &TaskInfo)> + '_ {
        self.tasks_timeline
            .get_worker_task_history(worker_id, SystemTime::now())
    }

    pub fn query_allocation_queues_at(
        &self,
        time: SystemTime,
    ) -> impl Iterator<Item = (&DescriptorId, &AllocationQueueInfo)> + '_ {
        self.alloc_timeline.get_queue_infos_at(time)
    }

    pub fn query_allocation_params(
        &self,
        descriptor_id: DescriptorId,
    ) -> Option<&AllocationQueueParams> {
        self.alloc_timeline.get_queue_params_for(&descriptor_id)
    }

    /// The Queued and Running allocations at `time` for a queue.
    pub fn query_allocations_info_at(
        &self,
        descriptor_id: DescriptorId,
        time: SystemTime,
    ) -> Option<impl Iterator<Item = (&AllocationId, &AllocationInfo)> + '_> {
        self.alloc_timeline
            .get_allocations_for_queue(descriptor_id, time)
    }

    pub fn query_worker_info_for(&self, worker_id: &WorkerId) -> Option<&WorkerConfiguration> {
        self.worker_timeline.get_worker_info_for(worker_id)
    }

    /// Calculates the number of workers connected to the cluster at the specified `time`.
    pub fn query_connected_worker_ids(
        &self,
        time: SystemTime,
    ) -> impl Iterator<Item = WorkerId> + '_ {
        self.worker_timeline.get_connected_worker_ids(time)
    }

    pub fn query_worker_overview_at(
        &self,
        worker_id: WorkerId,
        time: SystemTime,
    ) -> Option<&WorkerOverview> {
        self.worker_timeline.get_worker_overview_at(worker_id, time)
    }

    pub fn query_last_received_overviews(&self, time: SystemTime) -> Vec<&WorkerOverview> {
        self.worker_timeline.get_last_received_overviews(time)
    }
}

pub async fn create_data_fetch_process(
    refresh_interval: Duration,
    data: WrappedRcRefCell<DashboardData>,
    mut connection: ClientConnection,
) -> anyhow::Result<()> {
    let mut tick_duration = tokio::time::interval(refresh_interval);
    loop {
        let fetched_until = data.get().last_fetched_id();
        let events = fetch_events_after(&mut connection, fetched_until).await?;
        data.get_mut().update_data(events);
        tick_duration.tick().await;
    }
}

/// Gets the events from the server after the event_id specified
async fn fetch_events_after(
    connection: &mut ClientConnection,
    after_id: Option<u32>,
) -> crate::Result<Vec<MonitoringEvent>> {
    let response = rpc_call!(
        connection,
        FromClientMessage::MonitoringEvents (
            MonitoringEventRequest {
                after_id,
            }),
        ToClientMessage::MonitoringEventsResponse(response) => response
    )
    .await;
    response
}
