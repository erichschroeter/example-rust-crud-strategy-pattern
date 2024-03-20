use actix_web::{web, HttpResponse, Responder};
use tera::Context;

use super::{BACKEND_STRATEGY, VERSION};

pub async fn index(tmpl: web::Data<tera::Tera>) -> impl Responder {
    let mut ctx = Context::new();
    ctx.insert("version", &VERSION);
    ctx.insert("backend", &BACKEND_STRATEGY);
    ctx.insert("title", "Index Page");
    let s = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(s)
}
