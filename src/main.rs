mod assets;
#[cfg(not(target_arch = "wasm32"))]
mod command;
mod context;
mod editor;
mod game;
mod leaderboard;
mod local;
#[cfg(not(target_arch = "wasm32"))]
mod media;
mod menu;
mod model;
mod prelude;
mod render;
mod task;
mod ui;
mod util;

use context::Context;

use ctl_client::Nertboard;
use geng::prelude::*;

const FIXED_FPS: f64 = 60.0; // TODO: upgrade to 120 i think

const OPTIONS_STORAGE: &str = "options";
const HIGHSCORES_STORAGE: &str = "highscores";
const PLAYER_LOGIN_STORAGE: &str = "user";

const DISCORD_LOGIN_URL: &str = "https://discord.com/oauth2/authorize?client_id=1242091884709417061&response_type=code&scope=identify";

const DISCORD_SERVER_URL: &str = "https://discord.gg/Aq9bTvSbFN";

#[derive(clap::Parser)]
struct Opts {
    #[clap(long)]
    log: Option<String>,
    #[cfg(not(target_arch = "wasm32"))]
    #[command(subcommand)]
    command: Option<command::Command>,
    /// Skip intro screen.
    #[clap(long)]
    skip_intro: bool,
    /// Move through the level without player input.
    #[clap(long)]
    clean_auto: bool,
    #[clap(flatten)]
    geng: geng::CliArgs,
}

#[derive(geng::asset::Load, Deserialize, Clone)]
#[load(serde = "toml")]
pub struct Secrets {
    leaderboard: LeaderboardSecrets,
}

#[derive(Deserialize, Clone)]
pub struct LeaderboardSecrets {
    url: String,
}

fn main() {
    let opts: Opts = batbox::cli::parse();

    let mut builder = logger::builder();
    builder.filter_level(
        if let Some(level) = opts.log.as_deref().or(option_env!("LOG")) {
            match level {
                "debug" => log::LevelFilter::Debug,
                "info" => log::LevelFilter::Info,
                "warn" => log::LevelFilter::Warn,
                "error" => log::LevelFilter::Error,
                "off" => log::LevelFilter::Off,
                _ => panic!("invalid log level string"),
            }
        } else if cfg!(debug_assertions) {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        },
    );
    logger::init_with(builder).expect("failed to init logger");
    geng::setup_panic_handler();

    let mut options = geng::ContextOptions::default();
    options.window.title = "Geng Game".to_string();
    options.window.antialias = false;
    options.fixed_delta_time = 1.0 / FIXED_FPS;
    options.with_cli(&opts.geng);

    Geng::run_with(&options, move |geng| async move {
        let main = geng_main(geng, opts);

        #[cfg(not(target_arch = "wasm32"))]
        let main = async_compat::Compat::new(main);

        if let Err(err) = main.await {
            log::error!("{:?}", err);
        }
    });
}

async fn geng_main(geng: Geng, opts: Opts) -> anyhow::Result<()> {
    let loading_assets: Rc<assets::LoadingAssets> =
        geng::asset::Load::load(geng.asset_manager(), &run_dir().join("assets"), &())
            .await
            .context("when loading assets")?;

    let load_everything = load_everything(geng.clone());
    let loading_screen =
        menu::LoadingScreen::new(&geng, loading_assets, load_everything, opts.skip_intro).run();

    let (context, secrets, client) = loading_screen
        .await
        .ok_or_else(|| anyhow::Error::msg("loading screen failed"))??;

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(command) = opts.command {
        command
            .execute(context, secrets)
            .await
            .context("failed to execute the command")?;
        return Ok(());
    }

    // Main menu
    if opts.skip_intro {
        let leaderboard = leaderboard::Leaderboard::new(&geng, client.as_ref());
        let state = menu::LevelMenu::new(context, leaderboard);
        geng.run_state(state).await;
    } else {
        let state = menu::SplashScreen::new(context, client.as_ref());
        geng.run_state(state).await;
    }

    Ok(())
}

async fn load_everything(
    geng: Geng,
) -> anyhow::Result<(Context, Option<Secrets>, Option<Arc<Nertboard>>)> {
    let manager = geng.asset_manager();

    let assets = assets::Assets::load(manager).await?;
    let assets = Rc::new(assets);

    let secrets: Option<Secrets> =
        match geng::asset::Load::load(manager, &run_dir().join("secrets.toml"), &()).await {
            Ok(secrets) => {
                log::debug!("Successfully loaded secrets.toml");
                Some(secrets)
            }
            Err(err) => {
                log::debug!("Failed to load secrets.toml: {:?}", err);
                None
            }
        };
    let secrets = secrets.or_else(|| {
        let url = option_env!("LEADERBOARD_URL");
        if url.is_none() {
            log::debug!("LEADERBOARD_URL environment variable is not set, launching offline");
            return None;
        }
        log::debug!("Loaded LEADERBOARD_URL");
        Some(Secrets {
            leaderboard: LeaderboardSecrets {
                url: url?.to_string(),
            },
        })
    });
    let client = secrets
        .as_ref()
        .map(|secrets| ctl_client::Nertboard::new(&secrets.leaderboard.url))
        .transpose()?
        .map(Arc::new);
    if let Some(client) = &client {
        let _ = client.ping().await; // Ping the server to check if we are online
    }

    let context = context::Context::new(&geng, assets, client.as_ref())
        .await
        .expect("failed to initialize context");

    Ok((context, secrets, client))
}
