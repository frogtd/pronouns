use crate::mongo_id;
use actix_identity::Identity;
use actix_web::{get, post, web, Either, HttpResponse, Responder};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use askama::Template;
use mongodb::bson::doc;
use rand_core::OsRng;
use serde_derive::Deserialize;

#[derive(Template)]
#[template(path = "min/register.html")]
struct RegisterPage<'a> {
    title: &'a str,
    id: &'a str,
    error: Option<&'a str>,
}

#[derive(Deserialize, Debug)]

pub struct RegisterForm {
    email: String,
    username: String,
    password: String,
    repeat_password: String,
}

#[post("/register")]
pub async fn register_page(
    id: Identity,
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    form: web::Form<RegisterForm>,
) -> impl Responder {
    if form.repeat_password != form.password {
        let identity = match id.identity() {
            Some(x) => x,
            None => "".to_string(),
        };
        return Either::B(
            RegisterPage {
                title: "Register",
                id: &identity,
                error: Some("The repeated password is unequal."),
            }
            .render()
            .unwrap()
            .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
        );
    } else if form.password.len() < 8 {
        let identity = match id.identity() {
            Some(x) => x,
            None => "".to_string(),
        };
        return Either::B(
            RegisterPage {
                title: "Register",
                id: &identity,
                error: Some("The password is too short. Use a password over 8 characters long."),
            }
            .render()
            .unwrap()
            .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
        );
    }
    let collection = client
        .lock()
        .unwrap()
        .database("pronouns")
        .collection("accounts");

    if let Some(_x) = match collection
        .find_one(doc! { "username": form.username.clone() }, None)
        .await
    {
        Ok(x) => x,
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    } {
        // Username already exists
        let identity = match id.identity() {
            Some(x) => x,
            None => "".to_string(),
        };
        return Either::B(
            RegisterPage {
                title: "Register",
                id: &identity,
                error: Some("Username already exists."),
            }
            .render()
            .unwrap()
            .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
        );
    }

    // hash password
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);
    let hashed = match argon2.hash_password_simple(form.password.as_bytes(), salt.as_ref()) {
        Ok(x) => x.to_string(),
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };
    let identity = match collection
        .insert_one(
            doc! {
                "email": form.email.clone(),
                "confirmed_email": false,
                "username": form.username.clone(),
                "hash": hashed,
                "pronouns": Vec::<bson::oid::ObjectId>::new()
            },
            None,
        )
        .await
    {
        Ok(x) => x.inserted_id,
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };

    id.remember(mongo_id::MongoId::new(identity.as_object_id().unwrap().bytes()).to_string());
    Either::A(
        HttpResponse::SeeOther()
            .set_header(actix_web::http::header::LOCATION, "/")
            .finish(),
    )
}

#[get("/register")]
pub async fn register_request(id: Identity) -> impl Responder {
    let identity = match id.identity() {
        Some(x) => x,
        None => "".to_string(),
    };
    RegisterPage {
        title: "Register",
        id: &identity,
        error: None,
    }
    .render()
    .unwrap()
    .with_header(actix_web::http::header::CONTENT_TYPE, "text/html")
}
