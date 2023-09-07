use std::env;
use sqlx::{PgPool, postgres::PgPoolOptions};
use tokio::sync::OnceCell;
use color_eyre::Result;

#[derive(Debug)]
pub struct AppState {
    pub db: PgPool
}

static STATE: OnceCell<AppState> = OnceCell::const_new();
pub fn state() -> &'static AppState {
    STATE.get().unwrap()
}

async fn create_pool() -> Result<PgPool>  {
    Ok(PgPoolOptions::new()
        .connect(&env::var("DATABASE_URL")?)
        .await?)
}

pub async fn create_appstate() -> Result<()> {
    dotenvy::dotenv()?;
    Ok(STATE.set(AppState {
        db: create_pool().await?,
    })?)
}
