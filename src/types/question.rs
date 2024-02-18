use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuestionId(pub i32);

impl From<i32> for QuestionId {
    fn from(value: i32) -> Self {
        QuestionId(value)
    }
}

impl From<Option<i32>> for QuestionId {
    fn from(value: Option<i32>) -> Self {
        match value {
            Some(v) => QuestionId(v),
            None => QuestionId(0),
        }
    }
}

impl From<QuestionId> for i32 {
    fn from(value: QuestionId) -> Self {
        value.0
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Question {
    pub id: QuestionId,
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct NewQuestion {
    pub title: String,
    pub content: String,
    pub tags: Option<Vec<String>>,
}
