use crate::pronouns;
use actix_identity::Identity;
use actix_web::{get, Either, HttpResponse, Responder};
use askama::Template;
use futures::future;
use futures::stream::StreamExt;
use mongodb::bson::doc;

#[derive(Template)]
#[template(path = "min/index.html")]
struct IndexPage<'a> {
    examples: Vec<pronouns::Pronouns<'a>>,
    id: &'a str,
}

#[get("/")]
pub async fn index(
    client: actix_web::web::Data<std::sync::Mutex<mongodb::Client>>,
    id: Identity,
) -> impl Responder {
    // get pronouns collection
    let collection = client
        .lock()
        .unwrap()
        .database("pronouns")
        .collection("pronouns");

    let cursor = match collection
        .aggregate(
            vec![doc! {
                "$sample": {
                    "size": 3
                }
            }],
            None,
        )
        .await
    {
        Ok(x) => x,
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };
    let vec = cursor
        .filter(|doc| future::ready(doc.is_ok()))
        .map(|doc| doc.unwrap())
        .collect::<Vec<_>>()
        .await;
    let examples = vec
        .iter()
        .map(|doc| pronouns::from_doc!(&doc))
        .collect::<Vec<_>>();
    let identity = match id.identity() {
        Some(x) => x,
        None => "".to_string(),
    };
    Either::B(
        IndexPage {
            examples,
            id: &identity,
        }
        .render()
        .unwrap()
        .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
    )
}
