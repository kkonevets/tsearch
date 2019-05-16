#[derive(Queryable)]
pub struct Post {
    pub thread_id: i32,
    pub title: String,
    pub text: String,
    pub node_id: i32,
    pub needModer: i32,
    pub post_date: i32,
}
