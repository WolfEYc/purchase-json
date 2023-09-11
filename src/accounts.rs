use poem_openapi::{OpenApi, payload::Json, Object};
use sqlx::{types::chrono::NaiveDate, FromRow, QueryBuilder, Postgres, Execute};
use poem::{Result, http::StatusCode};
use crate::{state::state, PAGE_SIZE};
use tracing::info;

#[derive(Debug)]
pub struct AccountsApi;
#[OpenApi]
impl AccountsApi {
    #[oai(path = "/account", method = "post")]
    async fn read(&self, Json(filter): Json<AccountFilter>) -> Result<Json<Vec<Account>>> {
        let accounts = read(filter).await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(accounts))
    }
}

#[derive(Debug, FromRow, Object)]
pub struct Account {
    pub last_name: String,
    pub first_name: String,
    pub street_address: String,
    pub unit: Option<i16>,
    pub city: String,
    pub account_state: String,
    pub zip: i32,
    pub dob: NaiveDate,
    pub ssn: String,
    pub email_address: String,
    pub mobile_number: String,
    pub account_number: i64,
}

#[derive(Debug, Object)]
pub struct AccountFilter {
    pub account_number: Option<i64>,
    pub mobile_number: Option<i64>,
    pub email_address: Option<String>,
    pub ssn: Option<i32>,
    pub dob: Option<NaiveDate>,
    pub zip: Option<i32>,
    pub account_state: Option<String>,
    pub city: Option<String>,
    pub unit: Option<i32>,
    pub street_address: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub page: i64
}

pub async fn read(filter: AccountFilter) -> Result<Vec<Account>, sqlx::Error> {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM account WHERE ");

    if let Some(account_number) = filter.account_number {
        query.push("account_number = ").push_bind(account_number);
        let query = query.build_query_as();
        info!("{:?}", query.sql());

        let results = query.fetch_all(&state().db).await?;
        return Ok(results);
    };

    let mut seperated = query.separated(" AND ");
    if let Some(zip) = &filter.zip {
        seperated.push("zip = ").push_bind_unseparated(zip);
    }
    if let Some(unit) = &filter.unit {
        seperated.push("unit = ").push_bind_unseparated(*unit as i16);
    }
    if let Some(account_state) = &filter.account_state {
        seperated.push("account_state = ").push_bind_unseparated(account_state);
    }
    if let Some(mobile_number) = filter.mobile_number {
        seperated.push("mobile_number LIKE ").push_bind_unseparated(mobile_number.to_string()).push_unseparated(" || '%'");
    }
    if let Some(email_address) = &filter.email_address {
        seperated.push("LOWER(email_address) LIKE '%' || ").push_bind_unseparated(email_address.to_lowercase()).push_unseparated(" || '%'");
    }
    if let Some(ssn) = &filter.ssn {
        seperated.push("ssn LIKE '%' || ").push_bind_unseparated(ssn.to_string()).push_unseparated(" || '%'");
    }
    if let Some(city) = &filter.city {
        seperated.push("LOWER(city) LIKE '%' || ").push_bind_unseparated(city.to_lowercase()).push_unseparated(" || '%'");
    }
    if let Some(street_address) = &filter.street_address {
        seperated.push("LOWER(street_address) LIKE '%' || ").push_bind_unseparated(street_address.to_lowercase()).push_unseparated("|| '%'");
    }
    if let Some(last_name) = &filter.last_name {
        seperated.push("LOWER(last_name) LIKE ").push_bind_unseparated(last_name.to_lowercase()).push_unseparated(" || '%'");
    }
    if let Some(first_name) = &filter.first_name {
        seperated.push("LOWER(first_name) LIKE ").push_bind_unseparated(first_name.to_lowercase()).push_unseparated(" || '%'");
    }
    if query.sql().ends_with("WHERE ") {
        query = QueryBuilder::new("SELECT * FROM account ");
    }

    query.push(" ORDER BY ");
    let mut seperated = query.separated(", ");
    if let Some(dob) = &filter.dob {
        seperated.push("ABS(dob - ").push_bind_unseparated(dob).push_unseparated(") ASC");
    }
    if query.sql().ends_with("ORDER BY ") {
        query.push("last_name ASC");
    }
    query.push(format!(" LIMIT {} OFFSET ", PAGE_SIZE)).push_bind(filter.page * PAGE_SIZE);

    let query = query.build_query_as();
    info!("{:?}", query.sql());

    let results = query
        .fetch_all(&state().db)
        .await?;

    Ok(results)
}
