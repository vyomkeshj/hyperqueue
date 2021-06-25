use crate::{JobId, JobTaskId};
use std::num::NonZeroU64;

pub type ChannelId = u32;

pub struct StartTaskStreamMsg {
    job: JobId,
    task: JobTaskId,
    channel_names: Vec<String>,
}

pub struct DataMsg {
    job: JobId,
    task: JobTaskId,
    channel: ChannelId,
    data: Vec<u8>,
}

pub struct EndTaskStreamMsg {
    job: JobId,
    task: JobTaskId,
}

pub enum FromStreamerMessage {
    Start(StartTaskStreamMsg),
    Data(DataMsg),
    End(EndTaskStreamMsg),
}

pub struct EndTaskStreamResponseMsg {
    job: JobId,
    task: JobTaskId,
}

pub enum ToStreamerMessage {
    EndResponse(EndTaskStreamResponseMsg),
}
