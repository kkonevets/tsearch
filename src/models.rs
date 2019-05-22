#[derive(Queryable, Deserialize)]
pub struct Post {
    pub thread_id: i64,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub node_id: i64,
    #[serde(default)]
    pub needModer: i64,
    #[serde(default)]
    pub post_date: i64,
}
