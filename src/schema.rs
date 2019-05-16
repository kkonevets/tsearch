table! {
    threads_message_extra (thread_id) {
        thread_id -> Integer,
        title -> Varchar,
        text -> Text,
        node_id -> Integer,
        needModer -> Integer,
        post_date -> Integer,
    }
}
