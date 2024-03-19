
use actix_web::{web, HttpResponse, Responder};
use tera::Context;

use crate::crud::{UserStorage, csv::CsvUserStorage};

use super::VERSION;

pub async fn index(tmpl: web::Data<tera::Tera>, storage: web::Data<CsvUserStorage>) -> impl Responder {
    let users = storage.read_all().unwrap_or_default();
    let mut ctx = Context::new();
    ctx.insert("version", &VERSION);
    ctx.insert("title", "Index Page");
    ctx.insert("users", &users);
    let s = tmpl.render("index.html", &ctx).unwrap();
    HttpResponse::Ok().body(s)
}
