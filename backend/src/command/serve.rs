use actix_web::{rt, web, HttpServer};
use clap::ArgMatches;
use cor_args::{ArgHandler, ConfigHandler, DefaultHandler, EnvHandler, Handler};
use log::{debug, info};
use tera::Tera;

use crate::{
    cfg::{default_config_path, default_template_glob, Cfg},
    APP_PREFIX,
};

fn run_http_server(cfg: &Cfg) -> std::io::Result<()> {
    info!("Running HTTP Server at http://{}:{}", cfg.address, cfg.port);
    // let template_dir = cfg
    //     .template_dir
    //     .clone()
    //     .into_os_string()
    //     .into_string()
    //     .unwrap();
    // // let tera = Tera::new(&Path::new(&template_dir).join("/**/*").display().to_string()).unwrap();
    // // let template_dir = Arc::new(template_dir);
    // let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap();
    let tera = Tera::new(&cfg.template_glob).unwrap();
    let server = HttpServer::new(move || {
        actix_web::App::new()
            .app_data(web::Data::new(tera.clone()))
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
    // FUTURE add more parsing for new fields added to Cfg struct
    debug!("{}", cfg);
    match run_http_server(&cfg) {
        Ok(_) => {}
        Err(_) => {}
    }
}
