use diesel::prelude::*;

use uuid::Uuid;

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::teams)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Team {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub name: String,
    pub default_expiry: Option<i32>, // Default expiry in seconds
}

#[derive(Queryable, QueryableByName, Selectable, Insertable, AsChangeset, Clone)]
#[diesel(table_name = crate::schema::api_keys)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ApiKey {
    pub id: Uuid,
    pub created_at: chrono::NaiveDateTime,
    pub team_id: Uuid,
}
