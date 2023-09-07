use std::net::SocketAddr;
use poem::{listener::TcpListener, Route, Server};
use poem_openapi::OpenApiService;
use color_eyre::Result;
use purchase_json::{state::create_appstate, accounts::AccountsApi};

#[tokio::main]
async fn main() -> Result<()> {
    create_appstate().await?;
    let all_endpoins = AccountsApi;
    let api_service = OpenApiService::new(all_endpoins, "Purchase Api", "1.0");
    let ui = api_service.openapi_explorer();
    let app = Route::new().nest("/", api_service).nest("/docs", ui);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr);

    Server::new(listener)
        .run(app)
        .await?;

    Ok(())
}