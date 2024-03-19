use std::sync::Arc;

use actix_web::{rt, web, HttpServer};
use clap::ArgMatches;
use cor_args::{ArgHandler, ConfigHandler, DefaultHandler, EnvHandler, Handler};
use log::{debug, info};
use tera::Tera;

use crate::{
    cfg::{default_config_path, default_template_glob, Cfg}, crud::csv::CsvUserStorage, APP_PREFIX
};

fn run_http_server(cfg: &Cfg) -> std::io::Result<()> {
    info!("Running HTTP Server at http://{}:{}", cfg.address, cfg.port);

    let storage_strategy = match cfg.storage_strategy.as_str() {
        "csv" => {
            CsvUserStorage::new("users.csv")
        }
        _ => {
            eprintln!("Unknown storage strategy: {strategy}. Falling back to using '{strategy}' strategy.", strategy=cfg.storage_strategy);
            CsvUserStorage::new("users.csv")
        }
    };
    let storage_strategy: web::Data<CsvUserStorage> = web::Data::from(Arc::new(storage_strategy));
    let tera = Tera::new(&cfg.template_glob).unwrap();
    let server = HttpServer::new(move || {
        actix_web::App::new()
            .app_data(web::Data::new(tera.clone()))
            .app_data(storage_strategy.clone())
            .route("/", web::get().to(crate::route::index::index))
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
    let storage_strategy = ArgHandler::new(matches)
        .next(Box::new(
            EnvHandler::new().prefix(APP_PREFIX).next(Box::new(
                ConfigHandler::new(Box::new(
                    config::Config::builder()
                        .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
                        .build()
                        .unwrap_or_default(),
                ))
                .next(Box::new(DefaultHandler::new("csv"))),
            )),
        ))
        .handle_request("storage_strategy");
    if let Some(storage_strategy) = storage_strategy {
        cfg.storage_strategy = storage_strategy.to_owned();
    }

    debug!("{}", cfg);
    match run_http_server(&cfg) {
        Ok(_) => {}
        Err(_) => {}
    }
}
