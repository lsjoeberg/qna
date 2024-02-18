use serde::{Deserialize, Serialize};

use crate::types::question::QuestionId;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnswerId(pub i32);

impl From<i32> for AnswerId {
    fn from(value: i32) -> Self {
        AnswerId(value)
    }
}

impl From<AnswerId> for i32 {
    fn from(value: AnswerId) -> Self {
        value.0
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Answer {
    pub id: AnswerId,
    pub content: String,
    pub question_id: QuestionId,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NewAnswer {
    pub content: String,
    pub question_id: QuestionId,
}
