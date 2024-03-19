use actix_web::{web, HttpResponse, Responder};
use tera::Context;

use super::VERSION;

pub async fn index(tmpl: web::Data<tera::Tera>) -> impl Responder {
    // pub async fn index() -> impl Responder {
    // HttpResponse::Ok().body("Help text")
    let mut ctx = Context::new();
    // let version = env!("CARGO_PKG_VERSION").unwrap_or_else(|_| "unknown version".to_string());
    ctx.insert("version", &VERSION);
    ctx.insert("title", "Index Page");
    let s = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(s)
}
