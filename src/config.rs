pub struct Config {
    pub token: String,
    pub application_id: u64,
    pub guild_id: u64,
    pub ticket_category: u64,
    pub ticket_transcript: u64,
    pub app_log_channel: u64,
    pub app_results_channel: u64,
    pub db_url: String,
    pub channel_panel_ticket: u64,
    pub channel_panel_apply: u64,
    // --- BENVENUTO E RUOLI ---
    pub channel_welcome: u64,
    pub role_member: u64,
    pub role_vip: u64, 
    // --- REGOLAMENTO E DIRETTIVE ---
    pub channel_rules: u64,
    pub channel_omega: u64,
    pub channel_vantaggi_vip: u64,
    // --- SVILUPPO E DOSSIER ---
    pub channel_prezzi_bot: u64,
    pub channel_dossier: u64, 
    // --- STORIA E MANIFESTO ---
    pub channel_story: u64,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            token: std::env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN"),
            application_id: std::env::var("APPLICATION_ID").expect("Missing APPLICATION_ID").parse().expect("Invalid ID"),
            guild_id: std::env::var("GUILD_ID").expect("Missing GUILD_ID").parse().expect("Invalid ID"),
            ticket_category: std::env::var("TICKET_CATEGORY_ID").expect("Missing TICKET_CATEGORY_ID").parse().expect("Invalid ID"),
            ticket_transcript: std::env::var("TICKET_TRANSCRIPT_ID").expect("Missing TICKET_TRANSCRIPT_ID").parse().expect("Invalid ID"),
            app_log_channel: std::env::var("APP_LOG_CHANNEL_ID").expect("Missing APP_LOG_CHANNEL_ID").parse().expect("Invalid ID"),
            app_results_channel: std::env::var("APP_RESULTS_CHANNEL_ID").expect("Missing APP_RESULTS_CHANNEL_ID").parse().expect("Invalid ID"),
            db_url: std::env::var("DATABASE_URL").expect("Missing DATABASE_URL"),
            channel_panel_ticket: std::env::var("CHANNEL_PANEL_TICKET").expect("Missing CHANNEL_PANEL_TICKET").parse().expect("Invalid ID"),
            channel_panel_apply: std::env::var("CHANNEL_PANEL_APPLY").expect("Missing CHANNEL_PANEL_APPLY").parse().expect("Invalid ID"),
            
            channel_welcome: std::env::var("CHANNEL_WELCOME")
                .unwrap_or_else(|_| "0".to_string())
                .parse().unwrap_or(0),
                
            role_member: std::env::var("ROLE_MEMBER").expect("Missing ROLE_MEMBER").parse().expect("Invalid ID"),
            
            role_vip: std::env::var("ROLE_VIP").expect("Missing ROLE_VIP").parse().expect("Invalid ID"),

            channel_rules: std::env::var("CHANNEL_RULES").expect("Missing CHANNEL_RULES").parse().expect("Invalid ID"),

            channel_omega: std::env::var("CHANNEL_OMEGA").expect("Missing CHANNEL_OMEGA").parse().expect("Invalid ID"),

            channel_vantaggi_vip: std::env::var("CHANNEL_VANTAGGI_VIP").expect("Missing CHANNEL_VANTAGGI_VIP").parse().expect("Invalid ID"),

            channel_prezzi_bot: std::env::var("CHANNEL_PREZZI_BOT")
                .unwrap_or_else(|_| "0".to_string())
                .parse().unwrap_or(0),

            channel_dossier: std::env::var("CHANNEL_DOSSIER")
                .unwrap_or_else(|_| "0".to_string())
                .parse().unwrap_or(0),

            channel_story: std::env::var("CHANNEL_STORY")
                .unwrap_or_else(|_| "0".to_string())
                .parse().unwrap_or(0),
        }
    }
}
