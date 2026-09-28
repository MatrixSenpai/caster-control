use std::sync::Arc;
use anyhow::Context as _;
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
    body: Json<LiveConfig>,
    config: Data<AppConfig>,
    client: Data<obws::Client>,
) -> Result<impl Responder, Box<dyn std::error::Error>> {
    let mut new_body = body.into_inner();

    let team_one_name = new_body.team_one.name.clone().unwrap_or_default();
    let team_two_name = new_body.team_two.name.clone().unwrap_or_default();
    if new_body.scroll_items.is_empty() || !new_body.scroll_items.iter().any(|v| v.contains(" vs ")) {
        new_body.scroll_items.push(format!("{} vs {}", team_one_name, team_two_name))
    } else {
        new_body.scroll_items = new_body.scroll_items.iter_mut().map(|v| {
            if v.contains(" vs ") {
                format!("{} vs {}", team_one_name, team_two_name)
            } else { v.clone() }
        }).collect::<Vec<_>>();
    }

    let mut handle = config.live.lock().await;
    update_casters_obs(
        &client,
        new_body.caster_left.clone(), &handle.caster_left,
        new_body.caster_right.clone(), &handle.caster_right,
    ).await?;

    *handle = new_body;

    let dump = toml::to_string_pretty(&handle.clone())?;
    std::fs::write("./live_config.toml", &dump);

    info!("Config updated: {handle:?}, local dump success");
    Ok(actix_web::HttpResponse::Ok().finish())
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

async fn update_casters_obs(
    client: &Data<obws::Client>,
    left: CasterConfig, old_left: &CasterConfig,
    right: CasterConfig, old_right: &CasterConfig,
) -> anyhow::Result<()> {
    if left.ne(old_left) { sync_caster(client, &LEFT, &left).await?; }
    if right.ne(old_right) { sync_caster(client, &RIGHT, &right).await?; }

    Ok(())
}
async fn update_obs_call(client: &Data<obws::Client>, name: &str, settings: &serde_json::Value) -> anyhow::Result<()> {
    use obws::requests::inputs::{SetSettings, InputId};

    client.inputs().set_settings(
        SetSettings {
            input: InputId::Name(name),
            settings,
            overlay: Some(true),
        }
    ).await.context(format!("Could not update settings for {name}"))
}
async fn update_obs_visible(
    client: &Data<obws::Client>,
    scene: &str, source: &str,
    visible: bool
) -> anyhow::Result<()> {
    use obws::requests::{scenes::SceneId, scene_items::{Id, SetEnabled}};

    let item_id = client.scene_items().id(
        Id { scene: SceneId::Name(scene), source, search_offset: None }
    ).await.context(format!("Could not get scene with name {scene}"))?;

    client.scene_items().set_enabled(
        SetEnabled { scene: SceneId::Name(scene), item_id, enabled: visible }
    ).await.context(format!("Could not update scene with name {scene}"))?;

    Ok(())
}
async fn sync_caster(
    client: &Data<obws::Client>,
    side: &ObsData,
    data: &CasterConfig
) -> anyhow::Result<()> {
    let body = serde_json::json!(
        { "text": data.name }
    );
    update_obs_call(client, side.name, &body).await?;

    let body = serde_json::json!(
        { "url": data.vdo_link }
    );
    update_obs_call(client, side.cam, &body).await?;
    update_obs_call(client, side.audio, &body).await?;
    update_obs_visible(client, side.scene, side.cam, data.show_camera).await?;
    update_obs_visible(client, side.audio_scene, side.audio, data.show_camera).await?;
    update_obs_visible(client, side.scene, LOGO, !data.show_camera).await?;

    Ok(())
}