use crate::JobId;
use std::path::PathBuf;
use tako::server::rpc::ConnectionDescriptor;

pub enum StreamServerControlMessage {
    RegisterStream(JobId, PathBuf),
    UnregisterStream(JobId),
    AddConnection(ConnectionDescriptor),
}
