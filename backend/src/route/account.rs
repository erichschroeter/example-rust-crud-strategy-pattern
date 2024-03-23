use std::sync::Mutex;

use actix_web::{web, HttpResponse, Responder};
use common::Account;
use tera::Context;

use crate::crud::Crud;

use super::{BACKEND_STRATEGY, VERSION};

pub async fn list_accounts(
    tmpl: web::Data<tera::Tera>,
    storage: web::Data<Mutex<dyn Crud<Account>>>,
) -> impl Responder {
    if let Ok(storage) = storage.lock() {
        let accounts = storage.read_all().unwrap_or_default();
        let mut ctx = Context::new();
        ctx.insert("version", &VERSION);
        ctx.insert("backend", &BACKEND_STRATEGY);
        ctx.insert("title", "Index Page");
        ctx.insert("accounts", &accounts);
        let s = tmpl.render("accounts.html", &ctx).unwrap();
        HttpResponse::Ok().body(s)
    } else {
        HttpResponse::InternalServerError().finish()
    }
}

pub async fn create_account(
    account: web::Json<Account>,
    storage: web::Data<Mutex<dyn Crud<Account>>>,
) -> impl Responder {
    let response = &account.0;
    if let Ok(mut storage) = storage.lock() {
        if let Ok(_) = storage.create(&account) {
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
