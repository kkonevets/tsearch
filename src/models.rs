#[derive(Queryable, Deserialize)]
pub struct Post {
    pub thread_id: i64,
    pub title: String,
    pub text: String,
    pub node_id: i64,
    pub needModer: i64,
    pub post_date: i64,
}
