use crate::server::autoalloc::AllocationId;
use crate::server::autoalloc::DescriptorId;
use crate::transfer::messages::{AllocationQueueParams, JobDescription};
use crate::{JobId, JobTaskCount, TakoTaskId, WorkerId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tako::messages::common::WorkerConfiguration;
use tako::messages::gateway::LostWorkerReason;
use tako::messages::worker::WorkerOverview;
use tako::static_assert_size;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MonitoringEventPayload {
    /// New worker has connected to the server
    WorkerConnected(WorkerId, Box<WorkerConfiguration>),
    /// Worker has disconnected from the server
    WorkerLost(WorkerId, LostWorkerReason),
    /// Worker has proactively send its overview (task status and HW utilization report) to the server
    WorkerOverviewReceived(WorkerOverview),
    /// A Job was submitted by the user.
    JobCreated(JobId, Box<JobInfo>),
    /// All tasks of the job have finished.
    JobCompleted(JobId, DateTime<Utc>),
    /// Task has started to execute on some worker
    TaskStarted {
        task_id: TakoTaskId,
        worker_id: WorkerId,
    },
    /// Task has been finished
    TaskFinished(TakoTaskId),
    // Task that failed to execute
    TaskFailed(TakoTaskId),
    /// New allocation queue has been created
    AllocationQueueCreated(DescriptorId, Box<AllocationQueueParams>),
    /// Allocation queue has been removed
    AllocationQueueRemoved(DescriptorId),
    /// Allocation was submitted into PBS/Slurm
    AllocationQueued {
        descriptor_id: DescriptorId,
        allocation_id: AllocationId,
        worker_count: u64,
    },
    /// PBS/Slurm allocation started executing
    AllocationStarted(DescriptorId, AllocationId),
    /// PBS/Slurm allocation has finished executing
    AllocationFinished(DescriptorId, AllocationId),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JobInfo {
    pub name: String,
    pub job_desc: JobDescription,

    pub task_ids: Vec<TakoTaskId>,
    pub max_fails: Option<JobTaskCount>,
    pub log: Option<PathBuf>,

    pub submission_date: DateTime<Utc>,
    pub completion_date: Option<DateTime<Utc>>,
}

// Keep the size of the event structure in check
static_assert_size!(MonitoringEventPayload, 136);
