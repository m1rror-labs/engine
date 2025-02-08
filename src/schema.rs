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

table! {
    blocks (id) {
        id -> Uuid,
        created_at -> Timestamp,
        blockhash -> Bytea,
        previous_blockhash -> Bytea,
        parent_slot -> BigInt,
        block_height -> BigInt,
        slot -> BigInt,
        blockchain -> Uuid,
    }
}

table! {
    blockchain (id) {
        id -> Uuid,
        created_at -> Timestamp,
        airdrop_keypair -> Bytea,
    }
}

table! {
    transaction_account_keys (id) {
        id -> Uuid,
        created_at -> Timestamp,
        transaction_signature -> Bytea,
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
        transaction_signature -> Bytea,
        accounts -> Array<BigInt>,
        data -> Bytea,
        program_id -> Bytea,
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
        transaction_signature -> Bytea,
        err -> Nullable<Text>,
        compute_units_consumed -> BigInt,
        fee -> BigInt,
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
    transactions (id) {
        id -> Uuid,
        created_at -> Timestamp,
        signature -> Text,
        version -> Text,
        recent_blockhash -> Bytea,
        slot -> BigInt,
        blockchain -> Uuid,
    }
}
