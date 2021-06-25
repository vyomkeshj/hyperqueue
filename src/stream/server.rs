use crate::common::WrappedRcRefCell;
use crate::stream::control::StreamServerControlMessage;
use crate::transfer::stream::{FromStreamerMessage, ToStreamerMessage};
use crate::{JobId, Map};
use futures::stream::SplitStream;
use futures::{SinkExt, StreamExt};
use orion::aead::streaming::StreamOpener;
use std::path::PathBuf;
use tako::server::rpc::ConnectionDescriptor;
use tako::transfer::auth::{forward_queue_to_sealed_sink, open_message};
use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::{
    channel, unbounded_channel, Receiver, Sender, UnboundedReceiver, UnboundedSender,
};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

const STREAM_BUFFER_SIZE: usize = 32;

enum StreamMessage {
    Message(FromStreamerMessage),
    Close,
}

struct StreamServerState {
    streams: Map<JobId, Sender<StreamMessage>>,
    registrations: Map<JobId, PathBuf>,
}

impl StreamServerState {
    async fn get_stream(
        &mut self,
        job_id: JobId,
        message: StreamMessage,
    ) -> anyhow::Result<Sender<StreamMessage>> {
        if let Some(s) = self.streams.get(&job_id) {
            Ok(s.clone())
        } else {
            if let Some(path) = self.registrations.get(&job_id) {
                let (sender, receiver) = channel(STREAM_BUFFER_SIZE);
                let path = path.clone();
                tokio::task::spawn_local(async move { file_writer(receiver, path).await });
                Ok(sender)
            } else {
                anyhow::bail!("Job {} is not registered for streaming", job_id);
            }
        }
    }
}

type StreamServerStateRef = WrappedRcRefCell<StreamServerState>;

impl StreamServerStateRef {
    fn new() -> Self {
        WrappedRcRefCell::wrap(StreamServerState {
            streams: Default::default(),
            registrations: Default::default(),
        })
    }
}

async fn file_writer(receiver: Receiver<StreamMessage>, path: PathBuf) {
    let file = File::create(&path).await?;
}

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

async fn receive_loop(
    mut receiver: SplitStream<Framed<tokio::net::TcpStream, LengthDelimitedCodec>>,
    mut opener: Option<StreamOpener>,
) -> anyhow::Result<()> {
    while let Some(data) = receiver.next().await {
        let message: FromStreamerMessage = open_message(&mut opener, &data?)?;
        match message {
            FromStreamerMessage::Start(msg) => {}
            FromStreamerMessage::Data(msg) => {}
            FromStreamerMessage::End(msg) => {}
        }
    }
    Ok(())
}

async fn handle_connection(mut connection: ConnectionDescriptor) {
    /*let sender = async {};

    let receiver = async {};

    tokio::select! {
        () = sender => {}
        () = receiver => {}
    }*/
    /*    let opener = connection.opener.unwrap();
    let sealer = connection.sealer.unwrap();*/

    let (sender, receiver) = unbounded_channel();

    let snd_loop = forward_queue_to_sealed_sink(receiver, connection.sender, connection.sealer);

    tokio::select! {
        r = snd_loop => { log::debug!("Send queue for stream closed {:?}", r); },
        r = receive_loop(connection.receiver, connection.opener) => {
            log::debug!("Connection for stream closed {:?}", r);
        },
    }
}

async fn stream_server_main(
    mut control_receiver: UnboundedReceiver<StreamServerControlMessage>,
) -> () {
    /*let mut registrations: Map<StreamId, PathBuf> = Map::new();
    let mut streams: Map<StreamId, Sender<FromStreamerMessage>>;*/
    let state_ref = StreamServerStateRef::new();

    while let Some(msg) = control_receiver.recv().await {
        match msg {
            StreamServerControlMessage::RegisterStream(stream_id, path) => {
                let mut state = state_ref.get_mut();
                assert!(state.registrations.insert(stream_id, path).is_none());
            }
            StreamServerControlMessage::UnregisterStream(stream_id) => {
                let mut state = state_ref.get_mut();
                state.registrations.remove(&stream_id);
            }
            StreamServerControlMessage::AddConnection(connection) => {
                tokio::task::spawn_local(async move { handle_connection(connection).await });
            }
        }
    }
}
