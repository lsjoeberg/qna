use handle_errors::QueryError;
use sqlx::postgres::{PgPool, PgPoolOptions};

use crate::types::answer::{Answer, NewAnswer};
use crate::types::question::{NewQuestion, Question};

#[derive(Debug, Clone)]
pub struct Store {
    pub connection: PgPool,
}

impl Store {
    pub async fn new(url: &str) -> Self {
        let db_pool = match PgPoolOptions::new().max_connections(5).connect(url).await {
            Ok(pool) => pool,
            Err(err) => panic!("Failed to establish DB connection: {}", err),
        };
        Store {
            connection: db_pool,
        }
    }

    pub async fn get_questions(
        &self,
        limit: Option<i64>,
        offset: i64,
    ) -> Result<Vec<Question>, QueryError> {
        let questions: Result<Vec<Question>, sqlx::Error> = sqlx::query_as!(
            Question,
            r#"SELECT id, title, content, tags FROM questions LIMIT $1 OFFSET $2"#,
            limit,
            offset,
        )
        .fetch_all(&self.connection)
        .await;
        match questions {
            Ok(questions) => Ok(questions),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(QueryError::DataBaseQueryError)
            }
        }
    }

    pub async fn add_question(&self, new_question: NewQuestion) -> Result<Question, QueryError> {
        let question: Result<Question, sqlx::Error> = sqlx::query_as!(
            Question,
            r#"INSERT INTO questions (title, content, tags)
            VALUES  ($1, $2, $3)
            RETURNING id, title, content, tags"#,
            new_question.title,
            new_question.content,
            new_question.tags.as_deref(),
        )
        .fetch_one(&self.connection)
        .await;
        match question {
            Ok(q) => Ok(q),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(QueryError::DataBaseQueryError)
            }
        }
    }

    pub async fn update_question(
        &self,
        question: Question,
        question_id: i32,
    ) -> Result<Question, QueryError> {
        let question: Result<Question, sqlx::Error> = sqlx::query_as!(
            Question,
            r#"UPDATE questions
            SET title = $1, content = $2, tags = $3
            WHERE id = $4
            RETURNING id, title, content, tags"#,
            question.title,
            question.content,
            question.tags.as_deref(),
            question_id,
        )
        .fetch_one(&self.connection)
        .await;
        match question {
            Ok(q) => Ok(q),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(QueryError::DataBaseQueryError)
            }
        }
    }

    pub async fn delete_question(&self, question_id: i32) -> Result<bool, QueryError> {
        let result = sqlx::query!(r#"DELETE FROM questions WHERE id = $1"#, question_id,)
            .execute(&self.connection)
            .await;
        match result {
            Ok(_) => Ok(true),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(QueryError::DataBaseQueryError)
            }
        }
    }

    pub async fn add_answer(&self, new_answer: NewAnswer) -> Result<Answer, QueryError> {
        let answer = sqlx::query_as!(
            Answer,
            r#"INSERT INTO answers (content, corresponding_question)
            VALUES ($1, $2)
            RETURNING id, content, corresponding_question AS question_id"#,
            new_answer.content,
            new_answer.question_id.0,
        )
        .fetch_one(&self.connection)
        .await;
        match answer {
            Ok(answer) => Ok(answer),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(QueryError::DataBaseQueryError)
            }
        }
    }
}
