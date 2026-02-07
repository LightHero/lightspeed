use c3p0::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use strum::{AsRefStr, Display};

pub type OutboxMessageModel<D> = Record<OutboxMessageData<D>>;

/// Outbox message
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct OutboxMessageData<D> {
    /// The status of the outbox message
    pub(crate) status: OutboxMessageStatus,
    /// The type of the outbox message
    pub r#type: String,
    /// The number of retries in case of failure
    pub retries: u32,
    /// The payload of the outbox message
    pub payload: D,
}

/// Outbox message
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
pub struct OutboxMessage<D> {
    /// The number of retries in case of failure
    pub retries: u32,
    /// The payload of the outbox message
    pub payload: D,
}

impl<D> OutboxMessage<D> {
    /// Creates a new outbox message
    pub fn new(payload: D, retries: u32) -> Self {
        OutboxMessage { retries, payload }
    }

    pub fn to_data(self, r#type: String) -> OutboxMessageData<D> {
        OutboxMessageData { retries: self.retries, payload: self.payload, status: OutboxMessageStatus::Pending, r#type }
    }
}

impl<D: Send + Sync + Unpin + DeserializeOwned + Serialize> OutboxMessageData<D> {
    /// Creates a new outbox message
    pub fn new<S: Into<String>>(r#type: S, payload: D) -> Self {
        OutboxMessageData { status: OutboxMessageStatus::Pending, r#type: r#type.into(), retries: 0, payload }
    }

    /// Set the retries
    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries = retries;
        self
    }

    /// Returns the status
    pub fn status(&self) -> &OutboxMessageStatus {
        &self.status
    }
}

impl<D: Sized + Send + Sync + Unpin + DeserializeOwned + Serialize> DataType for OutboxMessageData<D> {
    const TABLE_NAME: &'static str = "LS_OUTBOX_MESSAGE";
    type CODEC = OutboxMessageDataCodec<D>;
}

/// Outbox message status
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, AsRefStr, Display)]
pub enum OutboxMessageStatus {
    Pending,
    Processing,
    Processed,
    Failed,
}

/// Outbox message codec for data versioning
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_codec_tag")]
pub enum OutboxMessageDataCodec<D> {
    V1(OutboxMessageData<D>),
}

impl<D: Send + Sync + DeserializeOwned + Serialize> Codec<OutboxMessageData<D>> for OutboxMessageDataCodec<D> {
    fn encode(data: OutboxMessageData<D>) -> Self {
        OutboxMessageDataCodec::V1(data)
    }

    fn decode(data: Self) -> OutboxMessageData<D> {
        match data {
            OutboxMessageDataCodec::V1(data) => data,
        }
    }
}
