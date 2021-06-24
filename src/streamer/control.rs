use crate::transfer::stream::StreamId;
use std::path::PathBuf;

pub enum StreamServerControlMessage {
    RegisterStream(StreamId, PathBuf),
    UnregisterStream(StreamId),
    AddConnection,
}
