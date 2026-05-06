use tokio::sync::oneshot;

use crate::error::AudioError;

pub(crate) enum Command {
    SetVolume {
        node_id: u32,
        channels: u32,
        volume: f32,
        reply: oneshot::Sender<Result<(), AudioError>>,
    },
    SetMute {
        node_id: u32,
        muted: bool,
        reply: oneshot::Sender<Result<(), AudioError>>,
    },
    SetDefaultOutput {
        node_name: String,
        reply: oneshot::Sender<Result<(), AudioError>>,
    },
    SetDefaultInput {
        node_name: String,
        reply: oneshot::Sender<Result<(), AudioError>>,
    },
    Quit,
}
