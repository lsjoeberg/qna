use handle_errors::Error;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::Row;
use tracing::event;

use crate::account::{Account, AccountId};
use crate::types::answer::{Answer, AnswerId, NewAnswer};
use crate::types::question::{NewQuestion, Question, QuestionId};

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
        let questions =
            sqlx::query(r#"SELECT id, title, content, tags FROM questions LIMIT $1 OFFSET $2"#)
                .bind(limit)
                .bind(offset)
                .map(|row: PgRow| Question {
                    id: QuestionId(row.get("id")),
                    title: row.get("title"),
                    content: row.get("content"),
                    tags: row.get("tags"),
                })
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

    pub async fn add_question(
        &self,
        new_question: NewQuestion,
        account_id: AccountId,
    ) -> Result<Question, Error> {
        let question = sqlx::query(
            r#"INSERT INTO questions (title, content, tags, account_id)
            VALUES  ($1, $2, $3, $4)
            RETURNING id, title, content, tags"#,
        )
        .bind(new_question.title)
        .bind(new_question.content)
        .bind(new_question.tags)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
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
        account_id: AccountId,
    ) -> Result<Question, Error> {
        let question = sqlx::query(
            r#"UPDATE questions
            SET title = $1, content = $2, tags = $3
            WHERE id = $4 AND account_id = $5
            RETURNING id, title, content, tags"#,
        )
        .bind(question.title)
        .bind(question.content)
        .bind(question.tags)
        .bind(question_id)
        .bind(account_id.0)
        .map(|row: PgRow| Question {
            id: QuestionId(row.get("id")),
            title: row.get("title"),
            content: row.get("content"),
            tags: row.get("tags"),
        })
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

    pub async fn delete_question(
        &self,
        question_id: i32,
        account_id: AccountId,
    ) -> Result<bool, Error> {
        let result = sqlx::query(r#"DELETE FROM questions WHERE id = $1 AND account_id = $2"#)
            .bind(question_id)
            .bind(account_id.0)
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

    pub async fn add_answer(
        &self,
        new_answer: NewAnswer,
        account_id: AccountId,
    ) -> Result<Answer, Error> {
        let answer = sqlx::query(
            r#"INSERT INTO answers (content, corresponding_question, account_id)
            VALUES ($1, $2, $3)
            RETURNING id, content, corresponding_question AS question_id"#,
        )
        .bind(new_answer.content)
        .bind(new_answer.question_id.0)
        .bind(account_id.0)
        .map(|row: PgRow| Answer {
            id: AnswerId(row.get("id")),
            content: row.get("content"),
            question_id: QuestionId(row.get("question_id")),
        })
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
        let result = sqlx::query(r#"INSERT INTO accounts (email, password) VALUES ($1, $2)"#)
            .bind(account.email)
            .bind(account.password)
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
        let account = sqlx::query(r#"SELECT * FROM accounts WHERE email = $1"#)
            .bind(email)
            .map(|row: PgRow| Account {
                id: Some(AccountId(row.get("id"))),
                email: row.get("email"),
                password: row.get("password"),
            })
            .fetch_one(&self.connection)
            .await;
        match account {
            Ok(account) => Ok(account),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(Error::DataBaseQueryError(err))
            }
        }
    }

    pub async fn is_question_owner(
        &self,
        question_id: i32,
        account_id: &AccountId,
    ) -> Result<bool, Error> {
        let question = sqlx::query(
            r#"SELECT id, title, content, tags FROM questions WHERE id = $1 AND account_id = $2"#,
        )
        .bind(question_id)
        .bind(account_id.0)
        .fetch_optional(&self.connection)
        .await;
        match question {
            Ok(question) => Ok(question.is_some()),
            Err(err) => {
                tracing::event!(tracing::Level::ERROR, "{:?}", err);
                Err(Error::DataBaseQueryError(err))
            }
        }
    }
}
