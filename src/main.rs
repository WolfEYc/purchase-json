use color_eyre::Result;
use poem::{listener::TcpListener, middleware::Cors, EndpointExt, Route, Server};
use poem_openapi::OpenApiService;
use purchase_json::{accounts::AccountsApi, purchases::PurchasesApi, state::create_appstate};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    create_appstate().await?;
    let all_endpoins = (AccountsApi, PurchasesApi);
    let api_service = OpenApiService::new(all_endpoins, "Purchase Api", "1.0");
    let ui = api_service.openapi_explorer();
    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        //.with(SetHeader::new().overriding(CACHE_CONTROL, HeaderValue::from_static("public, max-age=3600")))
        .with(Cors::new());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr);

    Server::new(listener).run(app).await?;

    Ok(())
}
