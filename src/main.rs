mod config;
mod commands;
mod handlers;
mod database;
mod utils;

use crate::config::Config;
use poise::serenity_prelude as serenity;
use ax_um::{routing::post, Json, Router, extract::State, http::StatusCode}; // Corretto typo se presente
use axum::{routing::post as post_route, Json as JsonEx, extract::State as StateEx};
use tower_http::cors::{Any, CorsLayer};
use serde::{Deserialize, Serialize};

pub struct Data {
    pub config: Config,
    pub db: sqlx::SqlitePool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

// Struttura dati aggiornata per ricevere il token dalla dashboard
#[derive(Deserialize)]
struct TicketUpdate {
    token: String,
    title: String,
    description: String,
}

// --- FUNZIONE DI VERIFICA OAUTH2 ---
async fn is_staff(token: &str, guild_id: u64, staff_role_id: u64) -> bool {
    let client = reqwest::Client::new();
    let url = format!("https://discord.com/api/v10/guilds/{}/members/@me", guild_id);

    match client.get(url).bearer_auth(token).send().await {
        Ok(res) => {
            if let Ok(member) = res.json::<serde_json::Value>().await {
                if let Some(roles) = member["roles"].as_array() {
                    let role_str = staff_role_id.to_string();
                    return roles.iter().any(|r| r.as_str() == Some(&role_str));
                }
            }
            false
        }
        Err(_) => false,
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    
    let config = Config::from_env(); 
    let token = config.token.clone();
    let app_id = config.application_id;
    let guild_id = config.guild_id;

    let db_pool = database::initialize_db(&config.db_url).await?;
    println!("Database Red Ghøst Army Mod inizializzato correttamente.");

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::general::ping(),
            commands::general::kick(),
            commands::general::ban(),
            commands::general::unban(),
            commands::general::timeout(),
            commands::general::untimeout(),
            commands::tickets::ticket(),
            commands::tickets::close(),
            commands::apply::apply(),
            commands::general::scontovip(),
            commands::general::dossier(),
        ],
        event_handler: |ctx, event, framework, data| {
            Box::pin(handlers::event_handler(ctx, event, framework, data))
        },
        ..Default::default()
    };

    let db_for_setup = db_pool.clone();
    let config_clone = config.clone();

    let framework = poise::Framework::builder()
        .options(options)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let g_id = serenity::GuildId::new(guild_id);
                let bot_id = ctx.http.get_current_user().await?.id;
                let image_path = "/home/discord/red_ghost_army_mod/assets/Red-Ghost-Army.jpg";

                poise::builtins::register_in_guild(ctx, &framework.options().commands, g_id).await?;

                // [ ... Qui rimangono tutti i tuoi invii automatici di regole, listini, ecc. ... ]
                // (Mantieni il codice originale che hai postato per i canali 1-8)

                println!("🚀 Red Ghøst Army Mod: Sistemi sincronizzati.");
                Ok(Data { config: config_clone, db: db_for_setup })
            })
        })
        .build();

    // --- AVVIO SERVER API (DASHBOARD) CON PROTEZIONE ---
    let http_client = framework.clone().http().clone();
    let db_web = db_pool.clone();
    let config_web = config.clone();

    tokio::spawn(async move {
        let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
        
        let app = Router::new()
            .route("/api/update-ticket", post(move |State(db): State<sqlx::SqlitePool>, Json(payload): Json<TicketUpdate>| {
                let http = http_client.clone();
                let conf = config_web.clone();
                
                async move {
                    // 1. VERIFICA SICUREZZA OAUTH2
                    if !is_staff(&payload.token, conf.guild_id, conf.role_staff).await {
                        println!("⚠️ Tentativo di accesso non autorizzato respinto.");
                        return StatusCode::UNAUTHORIZED;
                    }

                    // 2. ESECUZIONE COMANDO (SOLO SE STAFF)
                    let _ = sqlx::query!("INSERT OR REPLACE INTO web_configs (key, value) VALUES ('ticket_title', ?)", payload.title).execute(&db).await;
                    let _ = sqlx::query!("INSERT OR REPLACE INTO web_configs (key, value) VALUES ('ticket_desc', ?)", payload.description).execute(&db).await;
                    
                    let chan = serenity::ChannelId::new(conf.channel_panel_ticket);
                    if let Ok(msgs) = chan.messages(&http, serenity::GetMessages::new().limit(20)).await {
                        let bot_id = http.get_current_user().await.unwrap().id;
                        if let Some(m) = msgs.iter().find(|m| m.author.id == bot_id) {
                            let embed = serenity::CreateEmbed::new()
                                .title(&payload.title)
                                .description(&payload.description)
                                .color(0xFF0000)
                                .image("attachment://Red-Ghost-Army.jpg");
                                
                            let _ = m.channel_id.edit_message(&http, m.id, serenity::EditMessage::new().embed(embed)).await;
                        }
                    }
                    println!("✅ Dashboard: Deploy eseguito da operatore autorizzato.");
                    StatusCode::OK
                }
            }))
            .layer(cors)
            .with_state(db_web);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_MEMBERS;
    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .application_id(serenity::ApplicationId::new(app_id))
        .await?;

    client.start().await?;
    Ok(())
}

async fn clean_channel(ctx: &serenity::Context, channel_id: serenity::ChannelId) {
    if let Ok(messages) = channel_id.messages(ctx, serenity::GetMessages::new().limit(50)).await {
        for m in messages { let _ = m.delete(ctx).await; }
    }
}
