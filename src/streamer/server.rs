
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender, Sender};
use crate::streamer::control::StreamServerControlMessage;
use crate::Map;
use crate::transfer::stream::{StreamId, FromStreamerMessage};
use std::path::PathBuf;

pub fn start_stream_server() -> UnboundedSender<StreamServerControlMessage> {
    let (sender, receiver) = unbounded_channel();
    std::thread::spawn(|| {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(stream_server_main(receiver))
    });
    sender
}

async fn stream_server_main(mut control_receiver: UnboundedReceiver<StreamServerControlMessage>) -> () {
    let mut registrations : Map<StreamId, PathBuf> = Map::new();
    let mut streams: Map<StreamId, Sender<FromStreamerMessage>>;

    while let Some(msg) = control_receiver.recv().await {
        match msg {
            StreamServerControlMessage::RegisterStream(stream_id, path) => {
                assert!(registrations.insert(stream_id, path).is_none());
            }
            StreamServerControlMessage::UnregisterStream(stream_id) => {
                registrations.remove(&stream_id);
            }
            StreamServerControlMessage::AddConnection => {

            }
        }
    }
}