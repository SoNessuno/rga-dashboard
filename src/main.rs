mod config;
mod commands;
mod handlers;
mod database;
mod utils;

use crate::config::Config;
use poise::serenity_prelude as serenity;
use axum::{routing::post, Json, Router, extract::State, http::StatusCode};
use tower_http::cors::{Any, CorsLayer};

pub struct Data {
    pub config: Config,
    pub db: sqlx::SqlitePool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(serde::Deserialize)]
struct TicketUpdate {
    title: String,
    description: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv().ok();
    
    let config = Config::from_env(); 
    let token = config.token.clone();
    let app_id = config.application_id;
    let guild_id = config.guild_id;

    let db_pool = database::initialize_db(&config.db_url).await?;
    println!("Database Red Gh\u{00F8}st Army Mod inizializzato correttamente.");

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

                // --- 1. DIRETTIVA GENERALE RGA-01 ---
                let rules_chan = serenity::ChannelId::new(config_clone.channel_rules);
                let msgs = rules_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let rules_title = "\u{1F4DC} DIRETTIVA GENERALE RGA-01";
                let existing_rules = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(rules_title.to_string())));

                let rules_text = "## \u{1F6E1} PROTOCOLLO DI CONDOTTA - RED GH\u{00D8}ST ARMY\n\n\
                ### ARTICOLO I: GERARCHIA E DISIPLINA\n* **1.1 Rispetto del Comando:** Le decisioni dello Staff sono insindacabili.\n* **1.2 Etica Militare:** Linguaggio consono. Niente tossicit\u{00E0} o disturbo.\n\n\
                ### ARTICOLO II: SVILUPPO E SOFTWARE\n* **2.1 Tariffe:** Gli asset software partono da una base minima di **10\u{20AC}** per lo sviluppo.\n* **2.2 Hosting:** La messa online \u{00E8} a carico del richiedente.\n* **2.3 Propriet\u{00E0}:** La rivendita o manomissione degli asset RGA comporta l'espulsione.\n\n\
                ### ARTICOLO III: MANUTENZIONE\n* **3.1 Supporto:** Manutenzione gratuita per bug entro 7 giorni.\n* **3.2 Upgrade:** Ogni modifica successiva richiede un nuovo preventivo.\n\n\
                ### ARTICOLO IV: SICUREZZA E PRIVACY\n* **4.1 Riservatezza:** Vietata la diffusione di informazioni interne.\n* **4.2 Trattative:** Ogni operazione avviene esclusivamente tramite Ticket.\n\n\
                ### ARTICOLO V: PROTOCOLLO DOSSIER E SANZIONI\n* **5.1 Schedatura:** Gestione database per la sicurezza RGA.\n* **5.2 Conseguenze:** L'inserimento nel Dossier comporta Blacklist totale.\n* **5.3 Criteri:** Truffe, hacking, phishing o violazioni ToS.";

                let rules_embed = serenity::CreateEmbed::new().title(rules_title).description(rules_text).color(0x8B0000).image("attachment://Red-Ghost-Army.jpg").footer(serenity::CreateEmbedFooter::new("Revisione: 2026.5 | Red Gh\u{00F8}st Army Ops HQ"));

                match existing_rules {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(rules_embed)).await; }
                    None => { clean_channel(ctx, rules_chan).await; let _ = rules_chan.send_message(ctx, serenity::CreateMessage::new().embed(rules_embed).add_file(serenity::CreateAttachment::path(image_path).await?)).await; }
                }

                // --- 2. LISTINO PREZZI ---
                let prezzi_chan = serenity::ChannelId::new(config_clone.channel_prezzi_bot);
                let msgs = prezzi_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let prezzi_title = "\u{1F4B0} UNIT\u{00C0} SVILUPPO: LISTINO ASSET RGA";
                let existing_prezzi = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(prezzi_title.to_string())));

                let prezzi_text = "## \u{1F6E0} PROTOCOLLO FORNITURA SOFTWARE\n\n\
                ### \u{1F4E6} ASSET BASIC\n* **Costo:** A partire da **10\u{20AC}**\n* **Incluso:** Sviluppo completo delle funzioni concordate.\n\n\
                ### \u{1F3D7} ASSET CUSTOM\n* **Costo:** Valutazione su preventivo.\n\n\
                ### \u{1F4E1} HOSTING E MANTENIMENTO\n* **Nota:** Costo hosting separato dallo sviluppo.\n* **Servizio RGA:** Gestione diretta su nostri server disponibile.\n\n\
                ### \u{1F4C2} PROCEDURA\n1. Apri un Ticket.\n2. Invia briefing.\n3. Ricevi preventivo ufficiale.";

                let prezzi_embed = serenity::CreateEmbed::new().title(prezzi_title).description(prezzi_text).color(0x2ECC71).image("attachment://Red-Ghost-Army.jpg").footer(serenity::CreateEmbedFooter::new("Divisione R&D | Red Gh\u{00F8}st Army"));

                match existing_prezzi {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(prezzi_embed)).await; }
                    None => { clean_channel(ctx, prezzi_chan).await; let _ = prezzi_chan.send_message(ctx, serenity::CreateMessage::new().embed(prezzi_embed).add_file(serenity::CreateAttachment::path(image_path).await?)).await; }
                }

                // --- 3. PANNELLO TICKET ---
                let ticket_chan = serenity::ChannelId::new(config_clone.channel_panel_ticket);
                let msgs = ticket_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let ticket_title = "\u{1F3AB} Centro Supporto Red Gh\u{00F8}st Army";
                let existing_ticket = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(ticket_title.to_string())));

                let ticket_embed = serenity::CreateEmbed::new().title(ticket_title).description("Seleziona una categoria dal menu qui sotto per assistenza immediata.").color(0xFF0000).image("attachment://Red-Ghost-Army.jpg");
                let ticket_menu = serenity::CreateActionRow::SelectMenu(serenity::CreateSelectMenu::new("open_ticket_menu", serenity::CreateSelectMenuKind::String {
                    options: vec![
                        serenity::CreateSelectMenuOption::new("Supporto Tecnico", "tecnico").emoji(serenity::ReactionType::Unicode("\u{1F4BB}".to_string())),
                        serenity::CreateSelectMenuOption::new("Segnalazione", "segnalazione").emoji(serenity::ReactionType::Unicode("\u{26A0}".to_string())),
                        serenity::CreateSelectMenuOption::new("Domande Generali", "generale").emoji(serenity::ReactionType::Unicode("\u{2753}".to_string())),
                    ],
                }).placeholder("Scegli una categoria..."));

                match existing_ticket {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(ticket_embed).components(vec![ticket_menu])).await; }
                    None => { clean_channel(ctx, ticket_chan).await; let _ = ticket_chan.send_message(ctx, serenity::CreateMessage::new().embed(ticket_embed).add_file(serenity::CreateAttachment::path(image_path).await?).components(vec![ticket_menu])).await; }
                }

                // --- 4. DIRETTIVE OMEGA ---
                let omega_chan = serenity::ChannelId::new(config_clone.channel_omega);
                let msgs = omega_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let omega_title = "\u{1F4C2} CLASSIFIED: DIRETTIVE OMEGA";
                let existing_omega = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(omega_title.to_string())));

                let omega_text = "## \u{2623}\u{FE0F} DIRETTIVE OMEGA - LIVELLO 5\n\n* **D-01: Riservatezza Estrema.** Blacklist immediata per fuga dati.\n* **D-02: Gerarchia.** Ordini non discutibili.\n\n*Obiettivo: Supremazia Tecnologica.*";
                let omega_embed = serenity::CreateEmbed::new().title(omega_title).description(omega_text).color(0x000000).image("attachment://Red-Ghost-Army.jpg").footer(serenity::CreateEmbedFooter::new("SECURITY LEVEL: ALPHA-01"));

                match existing_omega {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(omega_embed)).await; }
                    None => { clean_channel(ctx, omega_chan).await; let _ = omega_chan.send_message(ctx, serenity::CreateMessage::new().embed(omega_embed).add_file(serenity::CreateAttachment::path(image_path).await?)).await; }
                }

                // --- 5. PANNELLO DOSSIER ---
                let dossier_chan = serenity::ChannelId::new(config_clone.channel_dossier);
                let msgs = dossier_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let dossier_title = "\u{1F5D2} ARCHIVIO DOSSIER NEMICI";
                let existing_dossier = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(dossier_title.to_string())));

                let dossier_panel = serenity::CreateEmbed::new().title(dossier_title).description("Usa `/dossier` per segnalare minacce. Ogni segnalazione verr\u{00E0} verificata.").color(0x1a1a1a).image("attachment://Red-Ghost-Army.jpg").footer(serenity::CreateEmbedFooter::new("SISTEMA DI ARCHIVIAZIONE CENTRALE RGA"));

                match existing_dossier {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(dossier_panel)).await; }
                    None => { clean_channel(ctx, dossier_chan).await; let _ = dossier_chan.send_message(ctx, serenity::CreateMessage::new().embed(dossier_panel).add_file(serenity::CreateAttachment::path(image_path).await?)).await; }
                }

                // --- 6. PANNELLO CANDIDATURE ---
                let apply_chan = serenity::ChannelId::new(config_clone.channel_panel_apply);
                let msgs = apply_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let apply_title = "\u{1F4C4} Reclutamento Red Gh\u{00F8}st Army";
                let existing_apply = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(apply_title.to_string())));

                let apply_embed = serenity::CreateEmbed::new().title(apply_title).description("Clicca il pulsante qui sotto per avviare il modulo di arruolamento.").color(0x00FF00);
                let apply_btn = serenity::CreateActionRow::Buttons(vec![serenity::CreateButton::new("start_apply").label("Inizia Candidatura").style(serenity::ButtonStyle::Success).emoji(serenity::ReactionType::Unicode("\u{1F4C4}".to_string()))]);

                match existing_apply {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(apply_embed).components(vec![apply_btn])).await; }
                    None => { clean_channel(ctx, apply_chan).await; let _ = apply_chan.send_message(ctx, serenity::CreateMessage::new().embed(apply_embed).components(vec![apply_btn])).await; }
                }

                // --- 7. VANTAGGI VIP ---
                let vip_chan = serenity::ChannelId::new(config_clone.channel_vantaggi_vip);
                let msgs = vip_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let vip_title = "\u{1F48E} ASSET VIP: VANTAGGI ESCLUSIVI";
                let existing_vip = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(vip_title.to_string())));

                let vip_embed = serenity::CreateEmbed::new().title(vip_title).description("## \u{26A1} PRIVILEGI ATTIVI\n* **Sconto Operativo:** 40% su ogni sviluppo.\n* **Priorit\u{00E0}:** Accesso immediato alle nuove release.\n* **Hosting:** Supporto prioritario infrastruttura RGA.").color(0xFFD700).image("attachment://Red-Ghost-Army.jpg");

                match existing_vip {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(vip_embed)).await; }
                    None => { clean_channel(ctx, vip_chan).await; let _ = vip_chan.send_message(ctx, serenity::CreateMessage::new().embed(vip_embed).add_file(serenity::CreateAttachment::path(image_path).await?)).await; }
                }

                // --- 8. MANIFESTO E STORIA ---
                let story_chan = serenity::ChannelId::new(config_clone.channel_story);
                let msgs = story_chan.messages(ctx, serenity::GetMessages::new().limit(10)).await?;
                let story_title = "\u{1F4D6} MANIFESTO OPERATIVO: RED GH\u{00D8}ST ARMY";
                let existing_story = msgs.iter().find(|m| m.author.id == bot_id && m.embeds.iter().any(|e| e.title == Some(story_title.to_string())));

                let story_text = "## \u{1F310} LE ORIGINI: IL FRONTE TELEGRAM\nNata dalle ombre di **Telegram**, la **Red Gh\u{00F8}st Army** ha mosso i primi passi come cellula d'elite...\n\n### \u{1F680} MISSIONE DISCORD\nL'approdo su Discord segna l'inizio della nostra espansione tecnologica.\n\n### \u{1F512} IL CODICE\nOgni linea di codice \u{00E8} un'arma.";
                let story_embed = serenity::CreateEmbed::new().title(story_title).description(story_text).color(0x000000).image("attachment://Red-Ghost-Army.jpg").footer(serenity::CreateEmbedFooter::new("DA TELEGRAM A DISCORD | NO COMPROMISE"));

                match existing_story {
                    Some(m) => { let _ = m.channel_id.edit_message(ctx, m.id, serenity::EditMessage::new().embed(story_embed)).await; }
                    None => { clean_channel(ctx, story_chan).await; let _ = story_chan.send_message(ctx, serenity::CreateMessage::new().embed(story_embed).add_file(serenity::CreateAttachment::path(image_path).await?)).await; }
                }

                println!("🚀 Red Gh\u{00F8}st Army Mod: Sistemi sincronizzati.");
                Ok(Data { config: config_clone, db: db_for_setup })
            })
        })
        .build();

    // --- AVVIO SERVER API (DASHBOARD) ---
    let http_client = framework.clone().http().clone();
    let db_web = db_pool.clone();
    let ticket_id = config.channel_panel_ticket;

    tokio::spawn(async move {
        let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
        let app = Router::new()
            .route("/api/update-ticket", post(move |State(db): State<sqlx::SqlitePool>, Json(payload): Json<TicketUpdate>| {
                let http = http_client.clone();
                async move {
                    let _ = sqlx::query!("INSERT OR REPLACE INTO web_configs (key, value) VALUES ('ticket_title', ?)", payload.title).execute(&db).await;
                    let _ = sqlx::query!("INSERT OR REPLACE INTO web_configs (key, value) VALUES ('ticket_desc', ?)", payload.description).execute(&db).await;
                    let chan = serenity::ChannelId::new(ticket_id);
                    if let Ok(msgs) = chan.messages(&http, serenity::GetMessages::new().limit(20)).await {
                        let bot_id = http.get_current_user().await.unwrap().id;
                        if let Some(m) = msgs.iter().find(|m| m.author.id == bot_id) {
                            let embed = serenity::CreateEmbed::new().title(&payload.title).description(&payload.description).color(0xFF0000);
                            let _ = m.channel_id.edit_message(&http, m.id, serenity::EditMessage::new().embed(embed)).await;
                        }
                    }
                    StatusCode::OK
                }
            }))
            .layer(cors).with_state(db_web);
        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    });

    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT | serenity::GatewayIntents::GUILD_MEMBERS;
    let mut client = serenity::ClientBuilder::new(token, intents).framework(framework).application_id(serenity::ApplicationId::new(app_id)).await?;

    client.start().await?;
    Ok(())
}

async fn clean_channel(ctx: &serenity::Context, channel_id: serenity::ChannelId) {
    if let Ok(messages) = channel_id.messages(ctx, serenity::GetMessages::new().limit(50)).await {
        for m in messages { let _ = m.delete(ctx).await; }
    }
}
