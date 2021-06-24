use crate::{JobId, JobTaskId};
use std::num::NonZeroU64;

pub type StreamId = NonZeroU64;
pub type ChannelId = u32;

pub struct TaskStreamIdentification {
    job: JobId,
    task: JobTaskId,
}

pub struct StartTaskStreamMsg {
    stream: StreamId,
    ts_id: TaskStreamIdentification,
    channel_names: Vec<String>,
}

pub struct DataMsg {
    stream: StreamId,
    ts_id: TaskStreamIdentification,
    channel: ChannelId,
    data: Vec<u8>,
}

pub struct EndTaskStreamMsg {
    stream: StreamId,
    ts_id: TaskStreamIdentification,
}

pub enum FromStreamerMessage {
    Start(StartTaskStreamMsg),
    Data(DataMsg),
    End(EndTaskStreamMsg)
}

pub struct EndTaskStreamResponseMsg {
    ts_id: TaskStreamIdentification
}

pub enum ToStreamerMessage {
    EndResponse(EndTaskStreamResponseMsg)
}
