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
use bytes::{BytesMut, BufMut};
use byteorder::{ByteOrder, LittleEndian};

const STREAM_BUFFER_SIZE: usize = 32;
const HQ_LOG_VERSION: u32 = 0;

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
                let (sender, mut receiver) = channel(STREAM_BUFFER_SIZE);
                self.streams.insert(job_id, sender.clone());
                let path = path.clone();
                tokio::task::spawn_local(async move {
                    if let Err(e) = file_writer(&mut receiver, path).await {
                        error_state(receiver, e.display());
                    }
                });
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

fn send_error()

async fn error_state(receiver: Receiver<StreamMessage>, message: String) {
    todo!()
}

async fn file_writer(receiver: &mut Receiver<StreamMessage>, path: PathBuf) -> anyhow::Result<()> {
    let mut file = File::create(&path).await?;
    let mut buffer = BytesMut::with_capacity(10);
    buffer.put_slice(b"hqlog/");
    buffer.put_u32::<LittleEndian>(HQ_LOG_VERSION);
    file.write_all(&header).await?;
    while let Some(msg) = receiver.recv().await {
        match msg {
            StreamMessage::Message(FromStreamerMessage::Start(s)) => {
                buffer.clear();
                buffer.put_u8::<LittleEndian>(0);
                buffer.put_u32::<LittleEndian>(s.task);
                if let Err(e) = file.write_all(&buffer).await {
                    return Err(e.into());
                }
            }
            StreamMessage::Message(FromStreamerMessage::Data(s)) => {
                buffer.clear();
                buffer.put_u8::<LittleEndian>(0);
                buffer.put_u32::<LittleEndian>(s.task);
            }
            StreamMessage::Close => break
        }
    }
    Ok(())
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
