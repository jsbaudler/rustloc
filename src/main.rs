use actix_web::{get, middleware::Logger, App, HttpResponse, HttpServer, Responder};
use csv;
use env_logger::Env;
use reqwest::Error;
use std::env;
use std::fs::File;
use std::io::Write;
use std::net::IpAddr;

#[get("/")]
async fn get_ip_address(request: actix_web::HttpRequest) -> impl Responder {
    let ip_address: Option<IpAddr> = request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|header_value| header_value.to_str().ok())
        .and_then(|x_forwarded_for| x_forwarded_for.split(',').next())
        .and_then(|ip_string| ip_string.parse().ok());

    let mut response_dict = std::collections::HashMap::new();
    response_dict.insert(
        "ip_address".to_string(),
        ip_address.map(|ip| ip.to_string()).unwrap_or_default(),
    );

    if let Some(ip) = ip_address {
        let ip_number = ip_to_number(&ip);
        response_dict.insert("ip_number".to_string(), ip_number.to_string());

        let ip_type = match ip {
            IpAddr::V4(_) => "IPv4",
            IpAddr::V6(_) => "IPv6",
        };
        response_dict.insert("ip_type".to_string(), ip_type.to_string());

        let country_code = match ip {
            IpAddr::V4(_) => lookup_ipv4_country(&ip_number),
            IpAddr::V6(_) => lookup_ipv6_country(&ip_number),
        };
        response_dict.insert("country_code".to_string(), country_code.clone());

        let is_eu = is_eu_country(&country_code);
        response_dict.insert("is_eu".to_string(), is_eu.to_string());
    }

    HttpResponse::Ok().json(response_dict)
}

#[get("/{ip_address}")]
async fn print_ip_address(ip_address: actix_web::web::Path<String>) -> impl Responder {
    let ip: Option<IpAddr> = ip_address.parse().ok();

    let mut response_dict = std::collections::HashMap::new();
    response_dict.insert(
        "ip_address".to_string(),
        ip.map(|ip| ip.to_string()).unwrap_or_default(),
    );

    if let Some(ip) = ip {
        let ip_number: u128 = match ip {
            IpAddr::V4(ip) => u32::from(ip).into(),
            IpAddr::V6(ip) => u128::from(ip),
        };

        response_dict.insert("ip_number".to_string(), ip_number.to_string());

        let ip_type = match ip {
            IpAddr::V4(_) => "IPv4",
            IpAddr::V6(_) => "IPv6",
        };
        response_dict.insert("ip_type".to_string(), ip_type.to_string());

        let country_code = match ip {
            IpAddr::V4(_) => lookup_ipv4_country(&ip_number),
            IpAddr::V6(_) => lookup_ipv6_country(&ip_number),
        };
        response_dict.insert("country_code".to_string(), country_code.clone());

        let is_eu = is_eu_country(&country_code);
        response_dict.insert("is_eu".to_string(), is_eu.to_string());
    }

    HttpResponse::Ok().json(response_dict)
}

async fn download_databases() -> Result<(), Error> {
    let ipv4_url =
        "https://cdn.jsdelivr.net/npm/@ip-location-db/asn-country/asn-country-ipv4-num.csv";
    let ipv6_url =
        "https://cdn.jsdelivr.net/npm/@ip-location-db/asn-country/asn-country-ipv6-num.csv";
    let ipv4_response = reqwest::get(ipv4_url).await?;
    let ipv6_response = reqwest::get(ipv6_url).await?;
    let mut ipv4_file = File::create("asn-country-ipv4-num.csv").expect("DOESNT WORK");
    let mut ipv6_file = File::create("asn-country-ipv6-num.csv").expect("DOESNT WORK");
    let _ = ipv4_file.write_all(&ipv4_response.bytes().await?);
    let _ = ipv6_file.write_all(&ipv6_response.bytes().await?);
    Ok(())
}

fn lookup_ipv4_country(ip_number: &u128) -> String {
    let file = File::open("asn-country-ipv4-num.csv").unwrap();
    let mut reader = csv::ReaderBuilder::new().delimiter(b',').from_reader(file);
    for result in reader.records() {
        let record = result.unwrap();
        let range_start: u128 = record[0].parse().unwrap();
        let range_end: u128 = record[1].parse().unwrap();
        if *ip_number >= range_start && *ip_number <= range_end {
            return record[2].to_string();
        }
    }
    "".to_string()
}

fn lookup_ipv6_country(ip_number: &u128) -> String {
    let file = File::open("asn-country-ipv6-num.csv").unwrap();
    let mut reader = csv::ReaderBuilder::new().delimiter(b',').from_reader(file);
    for result in reader.records() {
        let record = result.unwrap();
        let range_start: u128 = record[0].parse().unwrap();
        let range_end: u128 = record[1].parse().unwrap();
        if *ip_number >= range_start && *ip_number <= range_end {
            return record[2].to_string();
        }
    }
    "".to_string()
}

fn is_eu_country(country_code: &str) -> bool {
    match country_code {
        "AT" | "BE" | "BG" | "HR" | "CY" | "CZ" | "DK" | "EE" | "FI" | "FR" | "DE" | "GR"
        | "HU" | "IE" | "IT" | "LV" | "LT" | "LU" | "MT" | "NL" | "PL" | "PT" | "RO" | "SK"
        | "SI" | "ES" | "SE" => true,
        _ => false,
    }
}

fn ip_to_number(ip: &IpAddr) -> u128 {
    match ip {
        IpAddr::V4(v4) => u128::from(
            v4.clone()
                .octets()
                .iter()
                .rev()
                .fold(0, |acc, octet| (acc << 8) | u128::from(*octet)),
        ),
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            (u128::from(segments[0]) << 112)
                | (u128::from(segments[1]) << 96)
                | (u128::from(segments[2]) << 80)
                | (u128::from(segments[3]) << 64)
                | (u128::from(segments[4]) << 48)
                | (u128::from(segments[5]) << 32)
                | (u128::from(segments[6]) << 16)
                | u128::from(segments[7])
        }
    }
}

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