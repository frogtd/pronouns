use crate::mongo_id;
use actix_identity::Identity;
use actix_web::{get, post, web, Either, HttpResponse, Responder};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use askama::Template;
use mongodb::bson::doc;
use serde_derive::Deserialize;

#[derive(Template)]
#[template(path = "min/login.html")]
struct LoginPage<'a> {
    title: &'a str,
    id: &'a str,
    error: Option<&'a str>,
}

#[derive(Deserialize, Debug)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[post("/login")]
pub async fn login_page(
    id: Identity,
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    form: web::Form<LoginForm>,
) -> impl Responder {
    let collection = client
        .lock()
        .unwrap()
        .database("pronouns")
        .collection("accounts");
    let user_account = match collection
        .find_one(doc! { "username": form.username.clone() }, None)
        .await
    {
        Ok(Some(x)) => x,
        Ok(None) => {
            let identity = match id.identity() {
                Some(x) => x,
                None => "".to_string(),
            };
            return Either::B(
                LoginPage {
                    title: "Login",
                    id: &identity,
                    error: Some("Invalid username or password."),
                }
                .render()
                .unwrap()
                .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
            );
        }
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };
    let argon2 = Argon2::default();

    let parsed_hash =
        PasswordHash::new(user_account.get("hash").unwrap().as_str().unwrap()).unwrap();
    if argon2
        .verify_password(form.password.as_bytes(), &parsed_hash)
        .is_err()
    {
        let identity = match id.identity() {
            Some(x) => x,
            None => "".to_string(),
        };
        return Either::B(
            LoginPage {
                title: "Login",
                id: &identity,
                error: Some("Invalid username or password."),
            }
            .render()
            .unwrap()
            .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
        );
    }

    id.remember(
        mongo_id::MongoId::new(user_account.get_object_id("_id").unwrap().bytes()).to_string(),
    );
    Either::A(
        HttpResponse::SeeOther()
            .set_header(actix_web::http::header::LOCATION, "/")
            .finish(),
    )
}

#[get("/login")]
pub async fn login_request(id: Identity) -> impl Responder {
    let identity = match id.identity() {
        Some(x) => x,
        None => "".to_string(),
    };
    LoginPage {
        title: "Login",
        id: &identity,
        error: None,
    }
    .render()
    .unwrap()
    .with_header(actix_web::http::header::CONTENT_TYPE, "text/html")
}
