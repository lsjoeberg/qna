use warp::Filter;

struct Question {
    id: QuestionId,
    title: String,
    content: String,
    tags: Option<Vec<String>>,
}

struct QuestionId(String);

impl Question {
    fn new(id: QuestionId, title: String, content: String, tags: Option<Vec<String>>) -> Self {
        Question {
            id,
            title,
            content,
            tags,
        }
    }
}

#[tokio::main]
async fn main() {
    let hello = warp::get()
        .map(|| "Hello, World!");

    warp::serve(hello)
        .run(([127, 0, 0, 1], 7878))
        .await;
}
