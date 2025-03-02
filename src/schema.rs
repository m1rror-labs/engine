use diesel::{allow_tables_to_appear_in_same_query, table};

table! {
    accounts (id) {
        id -> Uuid,
        created_at -> Timestamp,
        address -> Varchar,
        lamports -> Numeric,
        data -> Bytea,
        owner -> Varchar,
        executable -> Bool,
        rent_epoch -> Numeric,
        label -> Nullable<Varchar>,
        blockchain -> Uuid,
    }
}

table! {
    blocks (id) {
        id -> Uuid,
        created_at -> Timestamp,
        blockchain -> Uuid,
        blockhash -> Bytea,
        previous_blockhash -> Bytea,
        parent_slot -> Numeric,
        block_height -> Numeric,
        slot -> Numeric,
    }
}

table! {
    blockchains (id) {
        id -> Uuid,
        created_at -> Timestamp,
        airdrop_keypair -> Bytea,
        team_id -> Uuid,
        label -> Nullable<Text>,
    }
}

table! {
    transactions (id) {
        id -> Uuid,
        created_at -> Timestamp,
        signature -> Text,
        version -> Text,
        recent_blockhash -> Bytea,
        slot -> Numeric,
        blockchain -> Uuid,
    }
}

allow_tables_to_appear_in_same_query!(
    transactions,
    transaction_account_keys,
    transaction_instructions,
    transaction_log_messages,
    transaction_meta,
    transaction_signatures,
    accounts,
    blocks,
    blockchains,
    teams,
    api_keys
);

table! {
    transaction_account_keys (id) {
        id -> Uuid,
        created_at -> Timestamp,
        transaction_signature -> Text,
        account -> Text,
        signer -> Bool,
        writable -> Bool,
        index -> SmallInt,
    }
}

table! {
    transaction_instructions (id) {
        id -> Uuid,
        created_at -> Timestamp,
        transaction_signature -> Text,
        accounts -> Array<SmallInt>,
        data -> Bytea,
        program_id -> Text,
        stack_height -> SmallInt,
        inner -> Bool,
    }
}

table! {
    transaction_log_messages (id) {
        id -> Uuid,
        created_at -> Timestamp,
        transaction_signature -> Text,
        log -> Text,
        index -> SmallInt,
    }
}

table! {
    transaction_meta (id) {
        id -> Uuid,
        created_at -> Timestamp,
        transaction_signature -> Text,
        err -> Nullable<Text>,
        compute_units_consumed -> Numeric,
        fee -> Numeric,
        pre_balances -> Array<BigInt>,
        post_balances -> Array<BigInt>,
    }
}

table! {
    transaction_signatures (id) {
        id -> Uuid,
        created_at -> Timestamp,
        transaction_signature -> Text,
        signature -> Text
    }
}

table! {
    teams (id) {
        id -> Uuid,
        created_at -> Timestamp,
        name -> Text,
    }
}

table! {
    api_keys (id) {
        id -> Uuid,
        created_at -> Timestamp,
        team_id -> Uuid,
        label -> Text,
    }
}
