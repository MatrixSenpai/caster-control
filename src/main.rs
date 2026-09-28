#![allow(unused, dead_code)]

mod config;
mod api;

#[macro_use]
extern crate log;

use std::ops::Deref;
use actix_web::Responder;
use actix_web::web::{Data, Json};
use anyhow::{Result, Context as _};
use crate::config::{AppConfig, LiveConfig};

#[actix_web::get("/")]
async fn index() -> impl Responder {
    actix_files::NamedFile::open("./dashboard.html")
}

#[actix_web::get("/info")]
async fn bar(data: Data<AppConfig>) -> impl Responder {
    actix_files::NamedFile::open(&data.local.broadcast_bar_file)
}

#[actix_web::get("/org-logo")]
async fn tls_logo() -> impl Responder {
    actix_files::NamedFile::open("./logos/tlslogofinalcropped.png")
}

#[tokio::main]
async fn main() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(std::time::SystemTime::now()),
                record.level(),
                record.target(),
                message
            ))
        })
        .level_for("caster_control", log::LevelFilter::Trace)
        .level(log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .chain(fern::log_file("./caster_control.log")?)
        .apply()?;

    let config = AppConfig::new();
    let obs_client = obws::Client::connect(
        "127.0.0.1", config.local.obs_port, config.local.obs_password.as_deref()
    ).await.context("Could not connect to OBS!")?;
    let wrapped_client = Data::new(obs_client);

    let client_handle = wrapped_client.clone();
    let server = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(Data::new(config.clone()))
            .app_data(client_handle.clone())
            .service(index)
            .service(bar)
            .service(tls_logo)
            .configure(api::api_routes)
    }).bind(("0.0.0.0", 0))?;

    let port = server
        .addrs()
        .first()
        .map(|addr| addr.port())
        .context("Server has no bound address")?;
    let url = format!("http://localhost:{port}");
    info!("Server bound: {url}");

    wrapped_client.inputs().set_settings(
        obws::requests::inputs::SetSettings {
            input: obws::requests::inputs::InputId::Name("Info Ticker"),
            settings: &serde_json::json!({
                "is_local_file": false,
                "url": format!("{url}/info")
            }),
            overlay: Some(true)
        }
    )
        .await.context("Could not update OBS settings!")?;
    info!("OBS is bound to new URL");

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        match webbrowser::open(&url) {
            Ok(()) => info!("Opened browser at {url}"),
            Err(e) => warn!("Could not open browser: {e}"),
        }
    });

    server.run().await.context("Server stopped with an error")?;
    Ok(())
}
