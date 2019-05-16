table! {
    threads_message_extra (thread_id) {
        thread_id -> BigInt,
        title -> Varchar,
        text -> Text,
        node_id -> BigInt,
        needModer -> BigInt,
        post_date -> BigInt,
    }
}
