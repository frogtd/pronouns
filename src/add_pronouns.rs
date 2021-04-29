use actix_web::{post, web, HttpResponse, Responder};
use mongodb::bson::doc;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct PronounsForm {
    subject: String,
    object: String,
    possessive_determiner: String,
    possessive_pronoun: String,
    reflexive: String,
}

/// POST `/pronouns/`
#[post("/pronouns")]
pub async fn list_add(
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    form: web::Form<PronounsForm>,
) -> impl Responder {
    let collection = client
        .lock()
        .unwrap()
        .database("pronouns")
        .collection("pronouns");

    match collection
        .insert_one(
            doc! {
                "subject": &form.subject,
                "object": &form.object,
                "possessive_determiner": &form.possessive_determiner,
                "possessive_pronoun": &form.possessive_pronoun,
                "reflexive": &form.reflexive,
                "popularity": 0 as i64,
            },
            mongodb::options::InsertOneOptions::default(),
        )
        .await
    {
        Ok(_) => (),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    HttpResponse::SeeOther()
        .set_header(actix_web::http::header::LOCATION, "/add")
        .finish()
}
