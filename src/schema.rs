use diesel::table;

table! {
    accounts (id) {
        id -> Uuid,
        created_at -> Timestamp,
        address -> Varchar,
        lamports -> BigInt,
        data -> Bytea,
        owner -> Varchar,
        executable -> Bool,
        rent_epoch -> BigInt,
        label -> Nullable<Varchar>,
        blockchain -> Uuid,
    }
}
