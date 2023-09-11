use poem_openapi::{OpenApi, payload::Json, Object};
use serde::Deserialize;
use sqlx::{types::{chrono::{NaiveDate, NaiveDateTime}, BigDecimal}, FromRow, QueryBuilder, Postgres, Execute};
use poem::{Result, http::StatusCode, web::Form};
use crate::{state::state, PAGE_SIZE};
use tracing::info;

#[derive(Debug)]
pub struct PurchasesApi;
#[OpenApi]
impl PurchasesApi {
    #[oai(path = "/purchase", method = "post")]
    async fn read(&self, Form(filter): Form<PurchaseFilter>) -> Result<Json<Vec<Purchase>>> {
        let purchases = read(filter).await
            .map_err(|e| poem::Error::from_string(e.to_string(), StatusCode::BAD_REQUEST))?;

        Ok(Json(purchases))
    }
}

#[derive(Debug, Object)]
pub struct Purchase {
    pub account_number: i64,
    pub purchase_datetime: NaiveDateTime,
    pub purchase_amount: String,
    pub post_date: NaiveDate,
    pub purchase_number: i32,
    pub merchant_number: String,
    pub merchant_name: String,
    pub merchant_state: String,
    pub merchant_category_code: i16,
}

impl From<PurchaseRow> for Purchase {
    fn from(value: PurchaseRow) -> Self {
        Purchase {
            account_number: value.account_number,
            purchase_datetime: value.purchase_datetime,
            purchase_amount: value.purchase_amount.to_string(),
            post_date: value.post_date, 
            purchase_number: value.purchase_number, 
            merchant_number: value.merchant_number, 
            merchant_name: value.merchant_name, 
            merchant_state: value.merchant_state, 
            merchant_category_code: value.merchant_category_code 
        }
    }
}

#[derive(Debug, FromRow)]
pub struct PurchaseRow {
    pub account_number: i64,
    pub purchase_datetime: NaiveDateTime,
    pub purchase_amount: BigDecimal,
    pub post_date: NaiveDate,
    pub purchase_number: i32,
    pub merchant_number: String,
    pub merchant_name: String,
    pub merchant_state: String,
    pub merchant_category_code: i16,
}

#[derive(Debug, Deserialize)]
pub struct PurchaseFilter {
    pub account_number: Option<i64>,
    pub purchase_datetime: Option<NaiveDateTime>,
    pub purchase_amount: Option<f64>,
    pub post_date: Option<NaiveDate>,
    pub purchase_number: Option<i32>,
    pub merchant_number: Option<String>,
    pub merchant_name: Option<String>,
    pub merchant_state: Option<String>,
    pub merchant_category_code: Option<i16>,
    pub page: i64
}

pub async fn read(filter: PurchaseFilter) -> color_eyre::Result<Vec<Purchase>> {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM purchase WHERE ");

    let mut seperated = query.separated(" AND ");
    if let Some(account_number) = &filter.account_number {
        seperated.push("account_number = ").push_bind_unseparated(account_number);
    }
    if let Some(purchase_number) = &filter.purchase_number {
        seperated.push("purchase_number = ").push_bind_unseparated(purchase_number);
    }
    if let Some(merchant_state) = &filter.merchant_state {
        seperated.push("merchant_state = ").push_bind_unseparated(merchant_state);
    }
    if let Some(merchant_category_code) = &filter.merchant_category_code {
        seperated.push("merchant_category_code = ").push_bind_unseparated(merchant_category_code);
    }
    if let Some(merchant_number) = &filter.merchant_number {
        seperated.push("LOWER(merchant_number) LIKE '%' || ").push_bind_unseparated(merchant_number.to_lowercase()).push_unseparated(" || '%'");
    }
    if let Some(merchant_name) = &filter.merchant_name {
        seperated.push("LOWER(merchant_name) LIKE '%' || ").push_bind_unseparated(merchant_name.to_lowercase()).push_unseparated(" || '%'");
    }
    
    if query.sql().ends_with("WHERE ") {
        query = QueryBuilder::new("SELECT * FROM purchase");
    }

    query.push(" ORDER BY ");
    let mut seperated = query.separated(", ");
    if let Some(purchase_datetime) = &filter.purchase_datetime {
        seperated.push("ABS(EXTRACT(EPOCH FROM (purchase_datetime - ").push_bind_unseparated(purchase_datetime).push_unseparated("))) ASC");
    }
    if let Some(post_date) = &filter.post_date {
        seperated.push("ABS(post_date - ").push_bind_unseparated(post_date).push_unseparated(") ASC");
    }
    if let Some(purchase_amount) = &filter.purchase_amount {
        seperated.push("ABS(purchase_amount - ").push_bind_unseparated(purchase_amount).push_unseparated(") ASC");
    }
    if query.sql().ends_with("ORDER BY ")  {
        query.push("purchase_datetime DESC");
    }

    query.push(format!(" LIMIT {} OFFSET ", PAGE_SIZE)).push_bind(filter.page * PAGE_SIZE);

    let query = query.build_query_as::<PurchaseRow>();
    info!("{:?}", query.sql());

    Ok(query.fetch_all(&state().db).await?
        .into_iter()
        .map(Purchase::from)
        .collect())
}
