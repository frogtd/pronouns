use crate::mongo_id;
use crate::pronouns;
use actix_identity::Identity;
use actix_web::{get, web, Either, HttpResponse, Responder};
use askama::Template;
use bson::doc;
use mongodb::options::FindOneOptions;

#[derive(Template)]
#[template(path = "min/get_pronoun.html")]
struct PronounPage<'a> {
    pronoun: pronouns::Pronouns<'a>,
    title: &'a str,
    id: &'a str,
    has_pronoun: bool,
}

#[get("/pronouns/{string_id}")]
pub async fn get_pronoun(
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    path_string_id: web::Path<(String,)>,
    id: Identity,
) -> impl Responder {
    let string_id = path_string_id.into_inner().0;
    let object_id = match mongo_id::str_to_object_id(&string_id) {
        Ok(x) => x,
        Err(_) => return Either::A(HttpResponse::NotFound().finish()),
    };
    let database = client.lock().unwrap().database("pronouns");
    let pronouns_collection = database.collection("pronouns");

    let find_options = FindOneOptions::builder().build();
    let doc = match pronouns_collection
        .find_one(doc! {"_id": object_id.clone() }, find_options)
        .await
    {
        Ok(Some(x)) => x,
        Ok(None) => return Either::A(HttpResponse::NotFound().finish()),
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };

    // `doc`s -> `Pronoun`s
    let pronoun = pronouns::from_doc!(doc);
    // let pronoun = pronouns::Pronouns::from_doc(doc);

    let identity = match id.identity() {
        Some(x) => x,
        None => "".to_string(),
    };
    let mut has_pronoun = false;
    if identity != "" {
        let identity_id = match mongo_id::str_to_object_id(&identity) {
            Ok(x) => x,
            Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
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
            Ok(None) => {
                id.forget();
                return Either::A(HttpResponse::NotFound().finish());
            }
            Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
        };
        has_pronoun = user_doc
            .get_array("pronouns")
            .unwrap()
            .contains(&bson::Bson::ObjectId(object_id))
    }
    Either::B(
        PronounPage {
            pronoun,
            title: &format!("{}/{}", pronoun.subject, pronoun.object),
            id: &identity,
            has_pronoun: has_pronoun,
        }
        .render()
        .unwrap()
        .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
    )
}
