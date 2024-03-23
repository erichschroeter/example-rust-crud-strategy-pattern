use std::sync::{Arc, Mutex};

use actix_web::{rt, web, HttpServer};
use clap::ArgMatches;
use common::Account;
use cor_args::{ArgHandler, ConfigHandler, DefaultHandler, EnvHandler, Handler};
use log::{debug, info};
use tera::Tera;

#[cfg(feature = "csv")]
use crate::crud::csv::CsvAccountStore;
#[cfg(feature = "sqlite")]
use crate::crud::sqlite::SqliteAccountStore;
use crate::{
    cfg::{default_config_path, default_template_glob, Cfg},
    crud::Crud,
    APP_PREFIX,
};

#[cfg(feature = "csv")]
fn create_store(cfg: &Cfg) -> Mutex<CsvAccountStore> {
    let storage_path = cfg
        .storage_path
        .to_owned()
        .unwrap_or("accounts.csv".to_string());
    Mutex::new(CsvAccountStore::new(&storage_path))
}

#[cfg(feature = "sqlite")]
fn create_store(cfg: &Cfg) -> Mutex<SqliteAccountStore> {
    let storage_path = cfg
        .storage_path
        .to_owned()
        .unwrap_or("accounts.sqlite".to_string());
    Mutex::new(SqliteAccountStore::new(&storage_path))
}

fn run_http_server(cfg: Cfg) -> std::io::Result<()> {
    info!("Running HTTP Server at http://{}:{}", cfg.address, cfg.port);
    let cfg_clone = cfg.clone();
    let tera = Tera::new(&cfg.template_glob).unwrap();
    let server = HttpServer::new(move || {
        let storage: Arc<Mutex<dyn Crud<Account>>> = Arc::new(create_store(&cfg_clone));
        actix_web::App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(web::Data::from(storage))
            .route("/", web::get().to(crate::route::index::index))
            .route("/account", web::get().to(crate::route::account::list_accounts))
            .route(
                "/account/create",
                web::post().to(crate::route::account::create_account),
            )
    })
    .bind((cfg.address.as_str(), cfg.port));

    if let Ok(server) = server {
        rt::System::new().block_on(server.run())
    } else {
        unimplemented!()
    }
}

pub fn serve(matches: &ArgMatches) {
    let config_path = ArgHandler::new(matches)
        .next(Box::new(EnvHandler::new().prefix(APP_PREFIX).next(
            Box::new(DefaultHandler::new(
                &default_config_path().display().to_string(),
            )),
        )))
        .handle_request("config");
    let config_path = config_path.expect("No config path");
    let mut cfg = Cfg::default();

    let template_glob = ArgHandler::new(matches)
        .next(Box::new(
            EnvHandler::new().prefix(APP_PREFIX).next(Box::new(
                ConfigHandler::new(Box::new(
                    config::Config::builder()
                        .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
                        .build()
                        .unwrap_or_default(),
                ))
                .next(Box::new(DefaultHandler::new(&default_template_glob()))),
            )),
        ))
        .handle_request("template_glob");
    if let Some(template_glob) = template_glob {
        cfg.template_glob = template_glob;
    }

    let address = ArgHandler::new(matches)
        .next(Box::new(
            EnvHandler::new().prefix(APP_PREFIX).next(Box::new(
                ConfigHandler::new(Box::new(
                    config::Config::builder()
                        .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
                        .build()
                        .unwrap_or_default(),
                ))
                .next(Box::new(DefaultHandler::new("0.0.0.0"))),
            )),
        ))
        .handle_request("address");
    if let Some(address) = address {
        cfg.address = address.to_owned();
    }

    let port = ArgHandler::new(matches)
        .next(Box::new(
            EnvHandler::new().prefix(APP_PREFIX).next(Box::new(
                ConfigHandler::new(Box::new(
                    config::Config::builder()
                        .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
                        .build()
                        .unwrap_or_default(),
                ))
                .next(Box::new(DefaultHandler::new("8080"))),
            )),
        ))
        .handle_request("port");
    if let Some(port) = port {
        cfg.port = port.parse::<u16>().expect(&format!(
            "Failed to convert {} to unsigned 16-bit integer",
            port
        ))
    }

    // Validate the Storage strategy here before calling `run_http_server` to avoid runtime erros there.
    if cfg!(feature = "csv") {
        let storage_path = ArgHandler::new(matches)
            .next(Box::new(
                EnvHandler::new().prefix(APP_PREFIX).next(Box::new(
                    ConfigHandler::new(Box::new(
                        config::Config::builder()
                            .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
                            .build()
                            .unwrap_or_default(),
                    ))
                    .next(Box::new(DefaultHandler::new(
                        std::env::current_dir()
                            .unwrap_or_default()
                            .join("accounts.csv")
                            .display()
                            .to_string()
                            .as_str(),
                    ))),
                )),
            ))
            .handle_request("storage_path");
        if let Some(storage_path) = storage_path {
            cfg.storage_path = Some(storage_path.to_owned());
        }
    } else if cfg!(feature = "sqlite") {
        let storage_path = ArgHandler::new(matches)
            .next(Box::new(
                EnvHandler::new().prefix(APP_PREFIX).next(Box::new(
                    ConfigHandler::new(Box::new(
                        config::Config::builder()
                            .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
                            .build()
                            .unwrap_or_default(),
                    ))
                    .next(Box::new(DefaultHandler::new(
                        std::env::current_dir()
                            .unwrap_or_default()
                            .join("accounts.sqlite")
                            .display()
                            .to_string()
                            .as_str(),
                    ))),
                )),
            ))
            .handle_request("storage_path");
        if let Some(storage_path) = storage_path {
            cfg.storage_path = Some(storage_path.to_owned());
        }
    }

    debug!("{}", cfg);
    match run_http_server(cfg) {
        Ok(_) => {}
        Err(_) => {}
    }
}
