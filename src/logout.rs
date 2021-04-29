use actix_identity::Identity;
use actix_web::{post, HttpResponse, Responder};

#[post("/logout")]
async fn logout(id: Identity) -> impl Responder {
    id.forget();
    HttpResponse::SeeOther()
        .set_header(actix_web::http::header::LOCATION, "/")
        .finish()
}
