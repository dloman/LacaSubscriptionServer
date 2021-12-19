use actix_web::{web, App, HttpServer, HttpResponse};
use braintree::{Address, Braintree, CreditCard, Customer, Environment};
use log::{info};
use serde::{Serialize, Deserialize};
use std::sync::{Mutex};

#[derive(Deserialize,Debug, Serialize)]
pub struct Signup {
    pub first_name : String,
    pub last_name : String,
    pub email : String,
    pub address : String,
    pub address2 : String,
    pub city : String,
    pub state : String,
    pub zip_code : String,
    pub payment_method_nonce : String,
    pub membership_type : String,
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn thanks() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/thanks.html"))
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn signup(signup : web::Form<Signup>, braintree : web::Data<Mutex<Braintree>>) -> HttpResponse {
    print!("request = {:#?}\n", signup);

    let braintree = &*(braintree.lock().unwrap());
    let result = braintree.customer().generate(Customer{
        email: Some(signup.email.to_string()),
        first_name: Some(signup.first_name.to_string()),
        last_name: Some(signup.last_name.to_string()),
        payment_method_nonce: Some(signup.payment_method_nonce.to_string()),
        credit_card: Some(CreditCard{
            billing_address: Some(Address{
                first_name: Some(signup.first_name.to_string()),
                last_name: Some(signup.last_name.to_string()),
                locality: Some(signup.city.to_string()),
                postal_code: Some(signup.zip_code.to_string()),
                region: Some(signup.state.to_string()),
                street_address: Some(signup.address.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    });
    print!("trying to print customer");
    match result {
        Ok(customer) => {
            let subscription = braintree.subscription().create(braintree::subscription::Request{
                plan_id: Some(signup.membership_type.to_string()),
                payment_method_token: customer.credit_card.unwrap().token,
            });

            match subscription {
                Ok(subscription) => println!("\n\nWooooo!!! {:#?} \n\n", subscription),
                Err(err) => println!("\nError: {}\n", err),
            }
        },
        Err(err) => println!("\nError: {}\n", err),
    }

    thanks().await
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/index.html"))
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn basic(braintree : web::Data<Mutex<Braintree>>) -> HttpResponse {
    form(braintree, "basic", 12, 2).await
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn middle(braintree : web::Data<Mutex<Braintree>>) -> HttpResponse {
    form(braintree, "super_cewl", 75, 25).await
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn top(braintree : web::Data<Mutex<Braintree>>) -> HttpResponse {
    form(braintree, "virtue_signaling", 31337, 30000).await
}

//----------------------------------------------------------------------------------------------------
//----------------------------------------------------------------------------------------------------
pub async fn form(braintree : web::Data<Mutex<Braintree>>, membership_type : &str, price : i16, discount : i16) -> HttpResponse {
    let braintree = &*(braintree.lock().unwrap());
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../static/form.html")
              .replace("MEMBERSHIPTYPE", membership_type)
              .replace("PRICE", format!("{}", price).as_str())
              .replace("DISCOUNT", format!("{}", discount).as_str())
              .replace("TOTAL", format!("{}", price - discount).as_str())
              .replace("CLIENT_TOKEN_FROM_SERVER", braintree.client_token().generate(Default::default()).unwrap().value.as_str()))
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
            .route("/basic", web::get().to(basic))
            .route("/middle", web::get().to(middle))
            .route("/top", web::get().to(top))
            .route("/thanks", web::get().to(thanks))
            .route("/signup", web::post().to(signup))
            .route("/", web::post().to(submit))
    })
    .bind("0.0.0.0:7777")?
        .run()
        .await
}
