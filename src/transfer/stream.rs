use crate::{JobId, JobTaskId};
use std::num::NonZeroU64;

pub type StreamId = NonZeroU64;
pub type ChannelId = u32;

pub struct StartStreamMessage {
    stream: StreamId,
    job: JobId,
    task: JobTaskId,
    channel_names: Vec<String>,
}

pub struct DataMessage {
    channel: ChannelId,
}

pub enum FromStreamerMessage {
    StartStream(StartStreamRequest),
    Data(),
}

pub enum ToStreamerMessage {}
