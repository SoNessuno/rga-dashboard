mod config;
mod commands;
mod handlers;
mod database;
mod utils;

use crate::config::Config;
use poise::serenity_prelude as serenity;
use axum::{routing::post, Json, Router, extract::State, http::StatusCode};
use tower_http::cors::{Any, CorsLayer};
use serde::Deserialize;
use std::sync::Arc;

pub struct Data {
    pub config: Config,
    pub db: sqlx::SqlitePool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Deserialize)]
struct TicketUpdate {
    token: String,
    title: String,
    description: String,
}

async fn is_staff(token: &str, guild_id: u64, staff_role_id: u64) -> bool {
    let client = reqwest::Client::new();
    let url = format!("https://discord.com/api/v10/users/@me/guilds/{}/member", guild_id);
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

async fn update_or_send(
    http: &Arc<serenity::Http>,
    channel_id: u64,
    embed: serenity::CreateEmbed,
    components: Option<Vec<serenity::CreateActionRow>>,
    file_path: Option<&str>,
) {
    let chan = serenity::ChannelId::new(channel_id);
    let bot_user = match http.get_current_user().await {
        Ok(u) => u,
        Err(_) => return,
    };

    if let Ok(msgs) = chan.messages(http, serenity::GetMessages::new().limit(15)).await {
        if let Some(m) = msgs.iter().find(|m: &&serenity::Message| m.author.id == bot_user.id) {
            let mut edit = serenity::EditMessage::new().embed(embed);
            if let Some(comp) = components { edit = edit.components(comp); }
            let _ = chan.edit_message(http, m.id, edit).await;
            return;
        }
    }

    let mut msg = serenity::CreateMessage::new().embed(embed);
    if let Some(comp) = components { msg = msg.components(comp); }
    if let Some(path) = file_path {
        if let Ok(att) = serenity::CreateAttachment::path(path).await {
            msg = msg.add_file(att);
        }
    }
    let _ = chan.send_message(http, msg).await;
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    let config = Config::from_env(); 
    let token = config.token.clone();
    let guild_id = config.guild_id;

    let db_pool = database::initialize_db(&config.db_url).await?;
    let db_web = db_pool.clone();
    let config_web = config.clone(); 
    let http_arc = Arc::new(serenity::Http::new(&token));

    let options = poise::FrameworkOptions {
        commands: vec![
            commands::general::ping(), commands::general::kick(), commands::general::ban(),
            commands::general::unban(), commands::general::timeout(), commands::general::untimeout(),
            commands::tickets::ticket(), commands::tickets::close(), commands::apply::apply(),
            commands::general::scontovip(), commands::general::dossier(),
        ],
        event_handler: |ctx, event, framework, data| Box::pin(handlers::event_handler(ctx, event, framework, data)),
        ..Default::default()
    };

    let db_setup = db_pool.clone();
    let config_setup = config.clone();
    let http_setup = Arc::clone(&http_arc);

    let framework = poise::Framework::builder()
        .options(options)
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                let g_id = serenity::GuildId::new(guild_id);
                poise::builtins::register_in_guild(ctx, &framework.options().commands, g_id).await?;
                let img = "/home/discord/red_ghost_army_mod/assets/Red-Ghost-Army.jpg";

                // --- 1. DIRETTIVA GENERALE RGA-01 ---
                update_or_send(&http_setup, config_setup.channel_rules, 
                    serenity::CreateEmbed::new().title("📜 DIRETTIVA GENERALE RGA-01")
                    .description("## 🛡️ PROTOCOLLO DI CONDOTTA - RED GHØST ARMY\n\n### ARTICOLO I: GERARCHIA E DISIPLINA\n* **1.1 Rispetto del Comando:** Ogni decisione presa dallo Staff è insindacabile.\n* **1.2 Etica Militare:** È richiesto un linguaggio consono. Ogni forma di tossicità o mancanza di rispetto verso i commilitoni comporterà sanzioni immediate.\n\n### ARTICOLO II: SVILUPPO E SOFTWARE\n* **2.1 Tariffe Asset:** Gli asset software (bot, script, dashboard) partono da una base minima di **10€**.\n* **2.2 Hosting:** La messa online e il mantenimento dell'host sono a carico del richiedente.\n\n### ARTICOLO III: MANUTENZIONE E GARANZIA\n* **3.1 Supporto:** Eventuali bug segnalati entro 7 giorni dalla consegna verranno fixati gratuitamente.\n* **3.2 Modifiche Post-Consegna:** Ogni modifica non concordata nel briefing iniziale comporterà un costo aggiuntivo.")
                    .color(0x8B0000).image("attachment://Red-Ghost-Army.jpg"), None, Some(img)).await;

                // --- 2. LISTINO ASSET RGA ---
                update_or_send(&http_setup, config_setup.channel_prezzi_bot, 
                    serenity::CreateEmbed::new().title("💰 LISTINO ASSET RGA")
                    .description("## 🛠️ PROTOCOLLO FORNITURA SOFTWARE\n\n### 📦 ASSET BASIC\n* **Costo:** A partire da **10€**\n* **Incluso:** Sviluppo completo delle funzioni concordate, file sorgenti e guida all'installazione.\n\n### 🏗️ ASSET CUSTOM\n* **Costo:** Valutazione su preventivo.\n* **Descrizione:** Progetti complessi, integrazioni API esterne o bot multifunzione.\n\n### 📡 PROCEDURA D'ACQUISTO\n1. Apri un Ticket nella categoria 'Acquisto Asset'.\n2. Invia il briefing dettagliato.\n3. Attendi la quotazione e l'approvazione del Comando.")
                    .color(0x2ECC71).image("attachment://Red-Ghost-Army.jpg"), None, Some(img)).await;

                // --- 3. PANNELLO TICKET (Dashboard Sync) ---
                let s_title = sqlx::query_as::<_, (String,)>("SELECT value FROM web_configs WHERE key = 'ticket_title'").fetch_optional(&db_setup).await.ok().flatten().map(|r| r.0).unwrap_or("📩 Centro Supporto RGA".into());
                let s_desc = sqlx::query_as::<_, (String,)>("SELECT value FROM web_configs WHERE key = 'ticket_desc'").fetch_optional(&db_setup).await.ok().flatten().map(|r| r.0).unwrap_or("Seleziona una categoria dal menu a tendina per ricevere assistenza immediata dal nostro team tecnico o segnalare una violazione.".into());
                let t_menu = vec![serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("open_ticket_menu", serenity::CreateSelectMenuKind::String { 
                    options: vec![
                        serenity::CreateSelectMenuOption::new("Supporto Tecnico", "tecnico").description("Problemi con asset o script").emoji('💻'),
                        serenity::CreateSelectMenuOption::new("Segnalazione", "segnalazione").description("Segnala un utente o un bug").emoji('⚠️'),
                        serenity::CreateSelectMenuOption::new("Acquisto Asset", "acquisto").description("Richiedi un nuovo software").emoji('💰'),
                        serenity::CreateSelectMenuOption::new("Domande Generali", "generale").description("Info varie").emoji('❓'),
                    ] 
                }).placeholder("Scegli la tua destinazione..."))];
                update_or_send(&http_setup, config_setup.channel_panel_ticket, serenity::CreateEmbed::new().title(s_title).description(s_desc).color(0xFF0000).image("attachment://Red-Ghost-Army.jpg"), Some(t_menu), Some(img)).await;

                // --- 4. DIRETTIVE OMEGA (CLASSIFIED) ---
                update_or_send(&http_setup, config_setup.channel_omega, 
                    serenity::CreateEmbed::new().title("📁 CLASSIFIED: DIRETTIVE OMEGA")
                    .description("## ☣️ LIVELLO 5 - ACCESSO LIMITATO\n\n* **D-01: Riservatezza Estrema.** Qualsiasi fuga di dati sensibili riguardanti gli asset interni comporterà la blacklist immediata e permanente.\n* **D-02: Gerarchia Inviolabile.** Gli ordini impartiti dai gradi superiori non sono discutibili.\n* **D-03: Neutralità.** L'esercito non prende parte a conflitti tra terze parti a meno di ordine diretto del comando.\n\n*Obiettivo: Supremazia Tecnologica e Operativa.*")
                    .color(0x000000).image("attachment://Red-Ghost-Army.jpg"), None, Some(img)).await;

                // --- 5. PANNELLO DOSSIER ---
                update_or_send(&http_setup, config_setup.channel_dossier, 
                    serenity::CreateEmbed::new().title("📒 ARCHIVIO DOSSIER")
                    .description("Usa il comando `/dossier` per segnalare minacce esterne, truffatori o violazioni gravi dei ToS di Discord. Ogni segnalazione verrà verificata e archiviata permanentemente nel database RGA per la sicurezza della community.")
                    .color(0x1a1a1a).image("attachment://Red-Ghost-Army.jpg"), None, Some(img)).await;

                // --- 6. PANNELLO CANDIDATURE ---
                let apply_btn = vec![serenity::CreateActionRow::Buttons(vec![serenity::CreateButton::new("start_apply").label("Inizia Candidatura").style(serenity::ButtonStyle::Success).emoji('📝')])];
                update_or_send(&http_setup, config_setup.channel_panel_apply, 
                    serenity::CreateEmbed::new().title("📝 RECLUTAMENTO RGA").description("Credi di avere le competenze per far parte dello Staff di Red Ghøst Army? Clicca il pulsante sotto per avviare il modulo di arruolamento.").color(0x00FF00), 
                    Some(apply_btn), None).await;

                // --- 7. VANTAGGI VIP ---
                update_or_send(&http_setup, config_setup.channel_vantaggi_vip, 
                    serenity::CreateEmbed::new().title("💎 VANTAGGI VIP")
                    .description("## ⚡ PRIVILEGI ATTIVI\n* **Sconto Operativo:** 40% di riduzione su ogni sviluppo software.\n* **Priorità Asset:** Accesso immediato alle release beta e alle nuove funzioni prima del pubblico.\n* **Hosting Support:** Supporto prioritario per la configurazione dei tuoi asset su VPS RGA.")
                    .color(0xFFD700).image("attachment://Red-Ghost-Army.jpg"), None, Some(img)).await;

                // --- 8. MANIFESTO E STORIA ---
                update_or_send(&http_setup, config_setup.channel_story, 
                    serenity::CreateEmbed::new().title("📖 MANIFESTO: RED GHØST ARMY")
                    .description("## 🌐 LE ORIGINI: IL FRONTE TELEGRAM\nNata dalle ombre di **Telegram**, la Red Ghøst Army è stata forgiata nel codice e nella necessità di sicurezza digitale. Quello che era un piccolo collettivo è diventato un'armata strutturata.\n\n### 🚀 MISSIONE\nL'approdo su Discord segna l'inizio della nostra espansione centralizzata. La nostra missione è fornire infrastrutture software impenetrabili e asset all'avanguardia per chiunque cerchi la perfezione tecnica.\n\n*Noi siamo il codice. Noi siamo l'ombra.*")
                    .color(0x000000).image("attachment://Red-Ghost-Army.jpg"), None, Some(img)).await;

                println!("🚀 Red Ghøst Army: Infrastruttura Sincronizzata e Operativa.");
                Ok(Data { config: config_setup, db: db_setup })
            })
        })
        .build();

    // Server API
    tokio::spawn(async move {
        let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
        let app = Router::new()
            .route("/api/update-ticket", post(move |State(db): State<sqlx::SqlitePool>, Json(payload): Json<TicketUpdate>| {
                let http = Arc::clone(&http_arc);
                let conf = config_web.clone();
                async move {
                    if !is_staff(&payload.token, conf.guild_id, conf.role_staff).await { return StatusCode::UNAUTHORIZED; }
                    let _ = sqlx::query("INSERT OR REPLACE INTO web_configs (key, value) VALUES ('ticket_title', ?), ('ticket_desc', ?)").bind(&payload.title).bind(&payload.description).execute(&db).await;
                    let t_menu = vec![serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("open_ticket_menu", serenity::CreateSelectMenuKind::String { 
                        options: vec![serenity::CreateSelectMenuOption::new("Supporto", "tecnico").emoji('💻')] 
                    }))];
                    update_or_send(&http, conf.channel_panel_ticket, serenity::CreateEmbed::new().title(&payload.title).description(&payload.description).color(0xFF0000), Some(t_menu), None).await;
                    StatusCode::OK
                }
            }))
            .layer(cors).with_state(db_web);
        
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    let intents = serenity::GatewayIntents::all();
    let mut client = serenity::ClientBuilder::new(token, intents).framework(framework).await?;
    client.start().await?;
    Ok(())
}
