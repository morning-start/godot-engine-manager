use clap::{Parser, Subcommand};
use gdem::core::config::ConfigTrait;
use gdem::core::source::Source;
use gdem::core::style;
use gdem::func::{config, install, list, remove, switch, sync};

#[derive(Parser)]
#[clap(
    name = "gdem",
    version = "1.3.0",
    about = "Godot Engine Manager is a Godot Engine version management tool developed based on the GodotHub.",
    after_help = "Before using, please first sync the data with `gdem sync`."
)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure the Godot Engine Manager.
    #[clap(name = "config", alias = "cfg")]
    Config {
        /// The source to use.
        #[clap(short, long)]
        source: Option<String>,
        /// The proxy to use.
        #[clap(short, long)]
        proxy: Option<String>,
    },
    /// Sync the data from GodotHub.
    #[clap(name = "sync", alias = "s")]
    Sync,
    /// List the local engines.
    #[clap(name = "list", alias = "ls")]
    List {
        /// List the remote engines.
        #[clap(short, long)]
        remote: bool,
        /// List the engine assets.
        #[clap(short, long)]
        version: Option<String>,
    },
    /// Install the engine.
    #[clap(name = "install", alias = "i")]
    Install {
        /// The engine version to install.
        /// Godot_v4.4.1-stable_mono_win64.zip
        engine: String,
        #[clap(short, long)]
        /// Force install.
        force: bool,
        #[clap(short = 'k', long)]
        /// Skip sha512 check.
        skip_check: bool,
    },
    /// Switch the engine.
    #[clap(name = "switch", alias = "sw")]
    Switch {
        /// The local engine to switch.
        /// Godot_v4.4.1-stable_mono_win64
        engine: String,
    },
    /// Remove the engine.
    #[clap(name = "remove", alias = "rm")]
    Remove {
        /// The local engine to remove.
        /// Godot_v4.4.1-stable_mono_win64
        engine: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Config { source, proxy } => {
            let mut cfg = config::Config::init();
            if let Some(source) = source {
                cfg.source = Source::from_str(source.as_str());
            }
            if let Some(proxy) = proxy {
                cfg.proxy = proxy;
            }
            config::link_appdata(&cfg.data);
            cfg.save();
        }
        Commands::Sync {} => {
            let cfg = config::Config::init();
            sync::sync_data(&cfg).await;
        }
        Commands::List { remote, version } => {
            let cfg = config::Config::init();
            // 如果都为None，则列出所有本地引擎
            if !remote && version.is_none() {
                let res = list::list_local_engines(&cfg.home).unwrap();
                let current = cfg.version.clone();
                let table = style::show_tree(&res, current.as_ref(), "Local Engines");
                println!("{}", table);
            } else if !remote && version.is_some() {
                let res =
                    list::list_remote_engine_assets(&cfg.data, version.as_ref().unwrap()).unwrap();
                let table = style::show_list(&res, "Remote Engine Assets");
                println!("{}", table);
            } else if remote && version.is_none() {
                let res = list::list_remote_engines(&cfg.data).unwrap();
                let table = style::show_list(&res, "Remote Engines");
                println!("{}", table);
            } else {
                let res =
                    list::list_remote_engines_major(&cfg.data, version.as_ref().unwrap()).unwrap();
                let table = style::show_list(&res, "Remote Engines");
                println!("{}", table);
            }
        }
        Commands::Install {
            engine,
            force,
            skip_check,
        } => {
            let cfg = config::Config::init();
            match install::full_install_process(&engine, &cfg, force, skip_check)
                .await
            {
                Ok(engine) => {
                    println!("Install success: {}", engine);
                }
                Err(msg) => {
                    eprintln!("Install failed: {}", msg);
                }
            };
        }
        Commands::Switch { engine } => {
            let mut cfg = config::Config::init();
            match switch::switch_engine(&engine, &mut cfg) {
                Ok(engine) => {
                    println!("Switch engine success: {}", engine);
                }
                Err(msg) => {
                    eprintln!("Switch engine failed: {}", msg);
                }
            };
        }
        Commands::Remove { engine } => {
            let mut cfg = config::Config::init();
            match remove::remove_engine(&engine, &mut cfg) {
                Ok(_) => {
                    println!("Remove engine success");
                }
                Err(msg) => {
                    eprintln!("Remove engine failed: {}", msg);
                }
            };
        }
    }
}
