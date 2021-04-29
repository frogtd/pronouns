#![feature(async_closure)]
// #![feature(pub_macro_rules)]
use actix_identity::Identity;
use actix_web::{get, web, Responder};
use askama::Template;
use mongodb::options::ClientOptions;
use mongodb::Client;
use std::sync::Mutex;
mod add_pronoun;
mod add_pronouns;
mod get_pronoun;
mod index_page;
mod login;
mod logout;
mod mongo_id;
mod profile;
mod pronouns;
mod pronouns_list;
mod register;

const IS_PROD: bool = false;

#[derive(Template)]
#[template(path = "min/add_pronoun.html")]
struct AddPage<'a> {
    id: &'a str,
    title: &'static str,
}

#[get("/add")]
async fn add_pronoun_page(id: Identity) -> impl Responder {
    let identity = match id.identity() {
        Some(x) => x,
        None => "".to_string(),
    };
    AddPage {
        id: &identity,
        title: "Add Pronoun",
    }
    .render()
    .unwrap()
    .with_header(actix_web::http::header::CONTENT_TYPE, "text/html")
}

/// Make sure to set a `MONGODB_URL` in `.env`
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{App, HttpServer};
    dotenv::dotenv().ok();

    // connect to db
    let mut client_options = ClientOptions::parse(&std::env::var("MONGODB_URL").unwrap())
        .await
        .unwrap();
    client_options.app_name = Some("PlantApi".to_string());
    let client = web::Data::new(Mutex::new(Client::with_options(client_options).unwrap()));
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    // server launch
    println!("Server launching...");
    HttpServer::new(move || {
        use actix_identity::{CookieIdentityPolicy, IdentityService};
        use actix_web::{
            http::ContentEncoding,
            middleware::{Compress, Logger},
        };
        App::new()
            .wrap(Logger::default())
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(
                    &hex::decode(std::env::var("SECRET_KEY").unwrap()).unwrap(),
                )
                .name("auth-cookie")
                .secure(IS_PROD)
                .max_age(i64::MAX),
            ))
            .wrap(Compress::new(ContentEncoding::Auto))
            .app_data(client.clone())
            .service(index_page::index)
            .service(add_pronouns::list_add)
            .service(add_pronoun::add_pronoun)
            .service(get_pronoun::get_pronoun)
            .service(add_pronoun_page)
            .service(register::register_page)
            .service(register::register_request)
            .service(profile::profile_page)
            .service(profile::me)
            .service(logout::logout)
            .service(login::login_request)
            .service(login::login_page)
            .service(pronouns_list::pronouns_list_page)
            .service(pronouns_list::pronouns_list_no_page)
    })
    .bind(format!("127.0.0.1:{}", &std::env::var("LOCAL_PORT").unwrap()))?
    .run()
    .await
}
