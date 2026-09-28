use std::sync::Arc;
use actix_web::{
    Responder,
    web::{scope, Data, Json, ServiceConfig}
};
use super::config::*;

pub fn api_routes(cfg: &mut ServiceConfig) {
    cfg
        .service(logo_file)
        .service(
            scope("/api")
                .service(live_config)
                .service(update_live_config)
                .service(league_logos)
                .service(obs_status)
        );
}

#[actix_web::get("/live")]
async fn live_config(config: Data<AppConfig>) -> impl Responder {
    let handle = config.live.lock().await;
    let handle = handle.clone();
    Json(handle)
}

#[actix_web::post("/live")]
async fn update_live_config(
    body: Json<LiveConfig>, config: Data<AppConfig>
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let mut handle = config.live.lock().await;
    *handle = body.into_inner();

    let dump = toml::to_string_pretty(&handle.clone())?;
    std::fs::write("./live_config.toml", &dump);

    info!("Config updated: {handle:?}, local dump success");

    Ok(actix_web::HttpResponse::Ok())
}

#[actix_web::get("/logos")]
async fn league_logos(config: Data<AppConfig>) -> impl Responder {
    let mut out = std::collections::BTreeMap::new();

    for (league, dir) in config.local.leagues.iter() {
        let mut files = std::fs::read_dir(dir)
            .into_iter().flatten().flatten()
            .filter(|e| e.metadata().map(|m| m.is_file()).unwrap_or(false))
            .filter_map(|e| e.file_name().into_string().ok())
            .filter(|n| {
                let n = n.to_lowercase();
                [".png", ".jpg", ".jpeg"].iter().any(|t| n.ends_with(t))
            })
            .collect::<Vec<_>>();
        files.sort();
        out.insert(league.clone(), files);
    }

    Json(out)
}

#[actix_web::get("/logos/{league}/{file}")]
async fn logo_file(
    path: actix_web::web::Path<(String, String)>,
    config: Data<AppConfig>,
) -> Result<actix_files::NamedFile, Box<dyn std::error::Error>> {
    let (league, file) = path.into_inner();
    let dir = config.local.leagues.get(&league).ok_or_else(|| actix_web::error::ErrorNotFound("unknown logo"))?;
    Ok(actix_files::NamedFile::open(std::path::Path::new(dir).join(&file))?)
}

#[actix_web::get("/obs")]
async fn obs_status(client: Data<obws::Client>) -> impl Responder {
    let version = client.general().version().await.map(|v| {
        format!("{}.{}.{}", v.obs_studio_version.major, v.obs_studio_version.minor, v.obs_studio_version.patch)
    });

    Json(serde_json::json!(
        {
            "connected": version.is_ok(),
            "version": version.unwrap_or_default()
        }
    ))
}