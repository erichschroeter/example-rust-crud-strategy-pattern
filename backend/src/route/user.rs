use std::sync::Mutex;

use actix_web::{web, HttpResponse, Responder};
use common::User;
use tera::Context;

use crate::crud::{csv::CsvUserStore, Crud};

use super::{BACKEND_STRATEGY, VERSION};

pub async fn list_users(
    tmpl: web::Data<tera::Tera>,
    storage: web::Data<Mutex<CsvUserStore>>,
) -> impl Responder {
    if let Ok(storage) = storage.lock() {
        let users = storage.read_all().unwrap_or_default();
        let mut ctx = Context::new();
        ctx.insert("version", &VERSION);
        ctx.insert("backend", &BACKEND_STRATEGY);
        ctx.insert("title", "Index Page");
        ctx.insert("users", &users);
        let s = tmpl.render("users.html", &ctx).unwrap();
        HttpResponse::Ok().body(s)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn create_user(
    user: web::Json<User>,
    storage: web::Data<Mutex<CsvUserStore>>,
) -> impl Responder {
    let response = &user.0;
    if let Ok(mut storage) = storage.lock() {
        if let Ok(_) = storage.create(&user) {
            HttpResponse::Ok().json(response)
        } else {
            log::error!("[C]RUD failed");
            HttpResponse::InternalServerError().finish()
        }
    } else {
        log::error!("Storage lock failed");
        HttpResponse::InternalServerError().finish()
    }
}
