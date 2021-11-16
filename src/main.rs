use actix_web::{web, App, HttpServer, HttpResponse};
use braintree::{Braintree, Environment};
use log::info;
use std::sync::{Mutex};

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn form(braintree : web::Data<Mutex<Braintree>>) -> HttpResponse {
    let braintree = &*(braintree.lock().unwrap());
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html").replace(
                "CLIENT_TOKEN_FROM_SERVER",
                braintree.client_token().generate(Default::default()).unwrap().value.as_str()))
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn submit(json : web::Json<serde_json::Value>) -> HttpResponse {
    HttpResponse::Ok().body(format!("submit: {:?}", &json))
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    info!("setting up braintree");

    info!("starting server on 7777!");
    HttpServer::new(move || {
        let merchant_id = std::env::var("MERCHANT_ID").expect("environment variable MERCHANT_ID is not defined");
        let braintree = web::Data::new(Mutex::new(Braintree::new(
                    Environment::Sandbox,
                    merchant_id,
                    std::env::var("PUBLIC_KEY").expect("environment variable PUBLIC_KEY is not defined"),
                    std::env::var("PRIVATE_KEY").expect("environment variable PRIVATE_KEY is not defined"),
                    )));
        App::new()
            .app_data(braintree)
            .service(actix_files::Files::new("/assets", "assets").show_files_listing())
            .route("/", web::get().to(index))
            .route("/basic", web::get().to(form))
            .route("/", web::post().to(submit))
    })
    .bind("0.0.0.0:7777")?
        .run()
        .await
}
