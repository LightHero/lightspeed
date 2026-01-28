use c3p0::*;
use serde::{Deserialize, Serialize};

pub type TaskModel = Record<TaskData>;

#[derive(Clone, Serialize, Deserialize)]
pub struct TaskData {
    pub token: String,
    pub username: String,
    pub token_type: TokenType,
    pub expire_at_epoch_seconds: i64,
}

impl DataType for TaskData {
    const TABLE_NAME: &'static str = "LS_OUTBOX_TASK";
    type CODEC = TaskDataCodec;
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TokenType {
    AccountActivation,
    ResetPassword,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "_codec_tag")]
pub enum TaskDataCodec {
    V1(TaskData),
}

impl Codec<TaskData> for TaskDataCodec {
    fn encode(data: TaskData) -> Self {
        TaskDataCodec::V1(data)
    }

    fn decode(data: Self) -> TaskData {
        match data {
            TaskDataCodec::V1(data) => data,
        }
    }
}
