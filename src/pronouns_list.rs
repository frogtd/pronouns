use crate::pronouns;
use actix_identity::Identity;
use actix_web::{get, web, Either, HttpResponse, Responder};
use askama::Template;
use bson::doc;
use futures::StreamExt;
use mongodb::options::FindOptions;

const PAGE_COUNT: i64 = 10;

#[derive(Template)]
#[template(path = "min/pronoun_list.html")]
struct ListPage<'a> {
    title: &'a str,
    id: &'a str,
    pronouns: Vec<pronouns::Pronouns<'a>>,
    page: i64,
    next_exists: bool,
}
#[get("/list")]
pub async fn pronouns_list_no_page(
    client: web::Data<std::sync::Mutex<mongodb::Client>>,
    id: Identity,
) -> impl Responder {
    pronouns_list_internal(client, 1, id).await
}
#[get("/list/{page}")]
pub async fn pronouns_list_page(
    client: web::Data<std::sync::Mutex<mongodb::Client>>,
    page: web::Path<(i64,)>,
    id: Identity,
) -> impl Responder {
    let page = page.into_inner().0;
    pronouns_list_internal(client, page, id).await
}
async fn pronouns_list_internal(
    client: web::Data<std::sync::Mutex<mongodb::Client>>,
    page: i64,
    id: Identity,
) -> impl Responder {
    if page <= 0 {
        return Either::A(HttpResponse::NotFound().finish());
    }
    let page = page - 1;
    let collection = client
        .lock()
        .unwrap()
        .database("pronouns")
        .collection("pronouns");
    let find_options = FindOptions::builder()
        .sort(doc! { "popularity" : -1 })
        .skip(page * PAGE_COUNT)
        .limit(PAGE_COUNT)
        .build();
    let mut cursor = match collection.find(None, find_options).await {
        Ok(x) => x,
        Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
    };
    let mut vec = Vec::new();
    while let Some(doc) = cursor.next().await {
        vec.push(match doc {
            Ok(x) => x,
            Err(_) => continue,
        })
    }
    if vec.len() == 0 {
        return Either::A(HttpResponse::NotFound().finish());
    }
    // this seems bad but i dont know a better way to do it
    let next_exists = {
        let find_options = FindOptions::builder()
            .sort(doc! { "popularity" : -1 })
            .skip((page + 1) * PAGE_COUNT)
            .limit(1)
            .build();
        let mut cursor = match collection.find(None, find_options).await {
            Ok(x) => x,
            Err(_) => return Either::A(HttpResponse::InternalServerError().finish()),
        };
        let mut vec = Vec::new();
        while let Some(doc) = cursor.next().await {
            vec.push(match doc {
                Ok(x) => x,
                Err(_) => continue,
            })
        }
        vec.len() == 0
    };
    let pronouns = vec
        .iter()
        .map(|doc| pronouns::from_doc!(&doc))
        .collect::<Vec<_>>();
    let identity = match id.identity() {
        Some(x) => x,
        None => "".to_string(),
    };
    Either::B(
        ListPage {
            title: "List",
            id: &identity,
            pronouns: pronouns,
            page: page + 1,
            next_exists,
        }
        .render()
        .unwrap()
        .with_header(actix_web::http::header::CONTENT_TYPE, "text/html"),
    )
}
