mod cfg;
mod command;
mod crud;
mod route;

#[cfg(all(feature = "csv", feature = "sqlite"))]
compile_error!("feature \"csv\" and feature \"sqlite\" cannot be enabled at the same time");

use cfg::default_config_path;
use clap::{value_parser, Arg};
use cor_args::{ArgHandler, ConfigHandler, DefaultHandler, EnvHandler, FileHandler, Handler};
use log::{debug, error, info, trace, warn, LevelFilter};
use std::path::PathBuf;

pub const APP_NAME: &str = "example-rust-crud-strategy-pattern";
pub const APP_PREFIX: &str = "example-rust-crud-strategy-pattern_";

/// Sets up logging based on the specified verbosity level.
///
/// This function initializes the logging framework using `env_logger` crate.
/// The verbosity level determines the amount of log output that will be displayed.
///
/// # Examples
///
/// ```
/// use crate::setup_logging;
///
/// setup_logging("debug");
/// ```
///
/// # Arguments
///
/// * `verbosity` - A string slice representing the desired verbosity level.
///   Valid values are "off", "error", "warn", "info", "debug", and "trace".
///   If an invalid value is provided, the default level will be set to "info".
///
/// # Dependencies
///
/// This function depends on the following crates:
///
/// - `env_logger` - For setting up logging.
/// - `log` - For defining log levels.
///
/// # Panics
///
/// This function will panic if the `verbosity` string cannot be parsed into a `LevelFilter`.
///
/// # Notes
///
/// It is recommended to call this function early in the program to set up logging
/// before any log messages are generated.
///
fn setup_logging(verbosity: &str) {
    env_logger::builder()
        .filter(None, verbosity.parse().unwrap_or(LevelFilter::Info))
        .init();

    error!("log level enabled: error");
    warn!("log level enabled: warn");
    info!("log level enabled: info");
    debug!("log level enabled: debug");
    trace!("log level enabled: trace");
}

// async fn index() -> impl Responder {
//     HttpResponse::Ok().body("Help text")
// }

// async fn upload(mut payload: Multipart) -> Result<HttpResponse, Error> {
//     while let Ok(Some(mut field)) = payload.try_next().await {
//         let content_disposition = field.content_disposition().unwrap();
//         let filename = content_disposition.get_filename().unwrap();
//         let filepath = format!("./tmp/{}", sanitize_filename::sanitize(&filename));
//         let mut f = web::block(|| std::fs::File::create(filepath)).await.unwrap();
//         while let Some(chunk) = field.next().await {
//             let data = chunk.unwrap();
//             f = web::block(move || f.write_all(&data).map(|_| f)).await?;
//         }
//     }
//     Ok(HttpResponse::Ok().into())
// }

struct App {
    args: clap::Command,
}

impl App {
    pub fn new() -> Self {
        App {
            args: clap::Command::new("example-rust-crud-strategy-pattern")
                .version("v1.0.0")
                .author("Your Name <your.email@example.com>")
                .about("FIXME")
                .arg(
                    Arg::new("config")
                        .short('c')
                        .long("config")
                        .value_name("FILE")
                        // .default_value(&default_config_path_value)
                        .help("Sets a custom config file")
                        .value_parser(value_parser!(PathBuf)),
                )
                .arg(
                    Arg::new("verbosity")
                        .short('v')
                        .long("verbosity")
                        .value_name("VERBOSE")
                        // .default_value(Cfg::default().verbosity)
                        .help("Sets the verbosity log level")
                        .long_help("Choices: [off, error, warn, info, debug, trace]"),
                )
                .infer_subcommands(true)
                .arg_required_else_help(true)
                .subcommand(
                    clap::Command::new("serve")
                        .about("Run the web server")
                        .arg(
                            Arg::new("address")
                                .long("address")
                                .short('a')
                                // .env("FIXME_address")
                                // .action(ArgAction::Set)
                                // .default_value("127.0.0.1")
                                .value_name("ADDRESS")
                                .help("The IP address to run the HTTP server on"),
                        )
                        .arg(
                            Arg::new("port")
                                .long("port")
                                .short('p')
                                // .env("FIXME_port")
                                // .action(ArgAction::Set)
                                // .default_value("8080")
                                // .value_parser(value_parser!(u16))
                                .value_name("PORT")
                                .help("The port to run the HTTP server on"),
                        )
                        .arg(
                            Arg::new("templates_dir")
                                .long("templates_dir")
                                .short('t')
                                .env("FIXME_templates_dir")
                                // .default_value(&default_template_dir)
                                .value_parser(value_parser!(PathBuf))
                                .value_name("DIR")
                                .help("Directory path to where HTML templates are stored"),
                        ),
                )
                .subcommand(
                    clap::Command::new("generate-manifest").about("Generates a manifest file"),
                ),
        }
    }

    pub fn run_with_args<I, T>(&mut self, args: I) -> Result<(), Box<dyn std::error::Error>>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let matches = &self.args.clone().get_matches_from(args);

        let config_path = ArgHandler::new(matches)
            .next(Box::new(EnvHandler::new().prefix(APP_PREFIX).next(
                Box::new(DefaultHandler::new(
                    &default_config_path().display().to_string(),
                )),
            )))
            .handle_request("config")
            .unwrap();

        let verbosity_handler = ArgHandler::new(matches).next(Box::new(
            EnvHandler::new().prefix(APP_PREFIX).next(Box::new(
                FileHandler::new(format!("~/.config/{APP_NAME}/verbosity")).next(Box::new(
                    ConfigHandler::new(Box::new(
                        config::Config::builder()
                            .add_source(config::File::new(&config_path, config::FileFormat::Yaml))
                            .build()
                            .unwrap_or_default(),
                    ))
                    .next(Box::new(DefaultHandler::new("info"))),
                    // ConfigHandler::new(config_path).next(Box::new(DefaultHandler::new("info"))),
                )),
            )),
        ));
        if let Some(verbosity) = verbosity_handler.handle_request("verbosity") {
            // std::env::set_var("RUST_LOG", "actix_web=debug");
            // std::env::set_var("RUST_LOG", "trace");
            // std::env::set_var("RUST_BACKTRACE", "1");
            setup_logging(&verbosity);
        }

        match matches.subcommand() {
            Some(("serve", sub_m)) => command::serve::serve(sub_m),
            subcommand => eprintln!("Invalid subcommand {:?}", subcommand),
        }
        Ok(())
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.run_with_args(std::env::args().into_iter())
    }
}

// #[actix_web::main]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    App::new().run()
}
