use actix_web::{middleware::Logger, App, HttpServer};
use env_logger::Env;
use std::env;
use rustloc::handlers::{get_ip_address, print_ip_address};
use rustloc::tooling::download_databases;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // initialize logging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    // download the databases
    let _ = download_databases().await;

    // configure the port
    let port_str: String = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
    let port = match port_str.parse::<u16>() {
        Ok(port) if (1..=65535).contains(&port) => port,
        _ => {
            panic!(
                "Invalid port number specified in SERVER_PORT environment variable: {}",
                port_str
            );
        }
    };

    // set the flapper version
    let rustloc_version = env::var("RUSTLOC_VERSION")
        .or_else(|_| env::var("CARGO_PKG_VERSION"))
        .unwrap_or_else(|_| "0.0.0-dev (not set)".to_string());

    // print out some basic info about the server
    log::info!("Starting Rustloc v{rustloc_version}");
    log::info!("Serving at 0.0.0.0:{port}");
    log::info!("Lookup your IP address at /");
    log::info!("Lookup any IP address at /<ip_address>");

    // start server
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(get_ip_address)
            .service(print_ip_address)
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}