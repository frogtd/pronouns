use crate::mongo_id;
use actix_identity::Identity;
use actix_web::{post, web, Either, HttpResponse, Responder};
use askama::Template;
use bson::doc;
use mongodb::options::FindOneOptions;
use serde_derive::Deserialize;

#[derive(Template)]
#[template(path = "min/pronoun_add.html")]
struct AddedPage<'a> {
    title: &'a str,
    id: &'a str,
}
#[derive(Template)]
#[template(path = "min/pronoun_remove.html")]
struct RemovePage<'a> {
    title: &'a str,
    id: &'a str,
}
#[derive(Deserialize, Debug)]
pub struct PronounForm {
    action: String,
}

#[post("/pronouns/{string_id}")]
pub async fn add_pronoun(
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    path_string_id: web::Path<(String,)>,
    form: web::Form<PronounForm>,
    id: Identity,
) -> impl Responder {
    // TODO: add/remove from popularity
    let string_id = path_string_id.into_inner().0;
    let object_id = match mongo_id::str_to_object_id(&string_id) {
        Ok(x) => x,
        Err(_) => return Either::A(HttpResponse::NotFound().finish()),
    };
    let identity = match id.identity() {
        Some(x) => x,
        None => return Either::A(HttpResponse::NotFound().finish()),
    };
    let identity_id = match mongo_id::str_to_object_id(&identity) {
        Ok(x) => x,
        Err(_) => return Either::A(HttpResponse::NotFound().finish()),
    };
    let database = client.lock().unwrap().database("pronouns");
    let account_collection = database.collection("accounts");
    let mut user_doc = match account_collection
        .find_one(
            doc! {"_id": identity_id.clone() },
            FindOneOptions::builder().build(),
        )
        .await
    {
        Ok(Some(x)) => x,
        Ok(None) => return Either::A(HttpResponse::NotFound().finish()),
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };
    let pronoun_collection = database.collection("pronouns");

    let mut pronoun_doc = match pronoun_collection
        .find_one(
            doc! {
                "_id": object_id.clone()
            },
            None,
        )
        .await
    {
        Ok(Some(x)) => x,
        Ok(None) => return Either::A(HttpResponse::InternalServerError().finish()),
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };
    match &form.action[..] {
        "add" => {
            if user_doc
                .get_array("pronouns")
                .unwrap()
                .contains(&bson::Bson::ObjectId(object_id.clone()))
            {
                return Either::A(
                    HttpResponse::SeeOther()
                        .set_header(
                            actix_web::http::header::LOCATION,
                            format!("/pronouns/{}", string_id),
                        )
                        .finish(),
                );
            }
            *pronoun_doc.get_i64_mut("popularity").unwrap() = match pronoun_doc.get_i64_mut("popularity").unwrap().checked_add(1) {
                Some(x) => x,
                None => *pronoun_doc.get_i64_mut("popularity").unwrap()
            };
            user_doc
                .get_array_mut("pronouns")
                .unwrap()
                .push(bson::Bson::ObjectId(object_id.clone()));
            match account_collection
                .update_one(doc! {"_id": identity_id }, user_doc, None)
                .await
            {
                Ok(_) => (),
                Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
            }
            match pronoun_collection
                .update_one(
                    doc! {
                        "_id": object_id
                    },
                    pronoun_doc,
                    None,
                )
                .await
            {
                Ok(_) => (),
                Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
            };

            return Either::B(
                AddedPage {
                    title: "Pronoun Added",
                    id: &identity,
                }
                .render()
                .unwrap()
                .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
            );
        }
        "remove" => {
            if let Some((index, _value)) = user_doc
                .get_array("pronouns")
                .unwrap()
                .iter()
                .enumerate()
                .find(|&x| x.1 == &bson::Bson::ObjectId(object_id.clone()))
            {
                user_doc.get_array_mut("pronouns").unwrap().remove(index);
                *pronoun_doc.get_i64_mut("popularity").unwrap() -= 1;

                match account_collection
                    .update_one(doc! {"_id": identity_id }, user_doc, None)
                    .await
                {
                    Ok(_) => (),
                    Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
                }
                match pronoun_collection
                    .update_one(
                        doc! {
                            "_id": object_id
                        },
                        pronoun_doc,
                        None,
                    )
                    .await
                {
                    Ok(_) => (),
                    Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
                };
                return Either::B(
                    RemovePage {
                        title: "Pronoun Removed",
                        id: &identity,
                    }
                    .render()
                    .unwrap()
                    .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
                );
            }

            return Either::A(HttpResponse::InternalServerError().finish());
        }
        _ => return Either::A(HttpResponse::InternalServerError().finish()),
    }
}
