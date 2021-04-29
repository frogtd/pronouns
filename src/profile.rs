use crate::mongo_id;
use crate::pronouns::{self, Pronouns};
use actix_identity::Identity;
use actix_web::{get, web, Either, HttpResponse, Responder};
use askama::Template;
use bson::doc;
use mongodb::options::FindOneOptions;

#[derive(Template)]
#[template(path = "min/profile.html")]
struct ProfilePage<'a> {
    title: &'a str,
    id: &'a str,
    user: User<'a>,
}

#[derive(Debug)]
struct User<'a> {
    username: &'a str,
    id: &'a str,
    pronouns: Vec<Pronouns<'a>>,
}

/// Redirect to /user/{username}
#[get("/me")]
pub async fn me(
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    id: Identity,
) -> impl Responder {
    let identity = match id.identity() {
        Some(x) => x,
        None => return HttpResponse::NotFound().finish(),
    };
    let database = client.lock().unwrap().database("pronouns");
    let identity_id = match mongo_id::str_to_object_id(&identity) {
        Ok(x) => x,
        Err(_) => return HttpResponse::NotFound().finish(),
    };

    let user_doc = match database
        .collection("accounts")
        .find_one(
            doc! {"_id": identity_id },
            FindOneOptions::builder().build(),
        )
        .await
    {
        Ok(Some(x)) => x,
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    HttpResponse::Found()
        .set_header(
            actix_web::http::header::LOCATION,
            format!("/user/{}", user_doc.get_str("username").unwrap()),
        )
        .finish()
}

#[get("/user/{username}")]
pub async fn profile_page(
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    id: Identity,
    username: web::Path<(String,)>,
) -> impl Responder {
    let identity = match id.identity() {
        Some(x) => x,
        None => return Either::A(HttpResponse::NotFound().finish()),
    };
    let database = client.lock().unwrap().database("pronouns");
    let username = username.into_inner().0;
    let accounts_collection = database.collection("accounts");
    let user_doc = match accounts_collection
        .find_one(
            doc! {"username": username },
            FindOneOptions::builder().build(),
        )
        .await
    {
        Ok(Some(x)) => x,
        Ok(None) => return Either::A(HttpResponse::NotFound().finish()),
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };
    let mut pronouns_vec = Vec::new();
    let user = {
        for pronoun in user_doc.get_array("pronouns").unwrap() {
            pronouns_vec.push(
                match database
                    .collection("pronouns")
                    .find_one(
                        doc! {"_id": pronoun.as_object_id().unwrap() },
                        FindOneOptions::builder().build(),
                    )
                    .await
                {
                    Ok(x) => x,
                    Err(_) => None,
                },
            );
        }
        User {
            username: user_doc.get_str("username").unwrap(),
            id: &mongo_id::MongoId::new(user_doc.get_object_id("_id").unwrap().bytes()).to_string(),
            pronouns: pronouns_vec
                .iter()
                .filter(|doc| doc.is_some())
                .map(|doc| {
                    let document = doc.as_ref().unwrap();
                    pronouns::from_doc!(document)
                })
                .collect(),
        }
    };
    Either::B(
        ProfilePage {
            title: "Register",
            id: &identity,
            user,
        }
        .render()
        .unwrap()
        .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
    )
}
