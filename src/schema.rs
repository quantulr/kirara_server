use diesel::table;

table! {
    users (id) {
        id -> BigInt,
        username -> Text,
        password -> Text,
        email -> Text
    }
}