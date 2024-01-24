use handle_errors::Error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use tracing::event;

use crate::account::{Account, AccountId};
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
    ) -> Result<Vec<Question>, Error> {
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
                Err(Error::DataBaseQueryError(err))
            }
        }
    }

    pub async fn add_question(&self, new_question: NewQuestion) -> Result<Question, Error> {
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
                Err(Error::DataBaseQueryError(err))
            }
        }
    }

    pub async fn update_question(
        &self,
        question: Question,
        question_id: i32,
    ) -> Result<Question, Error> {
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
                Err(Error::DataBaseQueryError(err))
            }
        }
    }

    pub async fn delete_question(&self, question_id: i32) -> Result<bool, Error> {
        let result = sqlx::query!(r#"DELETE FROM questions WHERE id = $1"#, question_id)
            .execute(&self.connection)
            .await;
        match result {
            Ok(_) => Ok(true),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(Error::DataBaseQueryError(err))
            }
        }
    }

    pub async fn add_answer(&self, new_answer: NewAnswer) -> Result<Answer, Error> {
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
                Err(Error::DataBaseQueryError(err))
            }
        }
    }

    pub async fn add_account(&self, account: Account) -> Result<bool, Error> {
        let result = sqlx::query!(
            r#"INSERT INTO accounts (email, password) VALUES ($1, $2)"#,
            account.email,
            account.password,
        )
        .execute(&self.connection)
        .await;
        match result {
            Ok(_) => Ok(true),
            Err(err) => {
                event!(
                    tracing::Level::ERROR,
                    code = err
                        .as_database_error()
                        .unwrap()
                        .code()
                        .unwrap()
                        .parse::<i32>()
                        .unwrap(),
                    db_message = err.as_database_error().unwrap().message(),
                    constraint = err.as_database_error().unwrap().constraint().unwrap(),
                );
                Err(Error::DataBaseQueryError(err))
            }
        }
    }

    pub async fn get_account(&self, email: String) -> Result<Account, Error> {
        let questions = sqlx::query(r#"SELECT * FROM accounts WHERE email = $1"#)
            .bind(email)
            .map(|row: PgRow| Account {
                id: Some(AccountId(row.get("id"))),
                email: row.get("email"),
                password: row.get("password"),
            }
                )
                .fetch_one(&self.connection)
                .await;
        match questions {
            Ok(account) => Ok(account),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(Error::DataBaseQueryError(err))
            }
        }
    }
}
