use home::home_dir;
use std::collections::HashMap;
use tokio::process::Command;
use tokio::signal;
use tokio::sync::mpsc::{channel, Sender};
// use tokio::time::{sleep, Duration};
use once_cell::sync::Lazy;
use hotwatch::{Hotwatch, Event};

const APP_NAME: &str = "Topaz";
const APP_VER: &str = "0.1.0";

static HOME_DIR: Lazy<String> = Lazy::new(|| home_dir().unwrap().to_str().unwrap().to_string());

static mut STATE: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut state = HashMap::new();
    state.insert("app_name".to_string(), APP_NAME.to_string());
    state.insert("app_ver".to_string(), APP_VER.to_string());
    state
});

fn help() {
    println!("Usage: topaz [opts] [filepath]\n");
    println!("Options or paths specified inline will override those in the config file.");
    println!("Options:");
    println!("  -h, --help\t\tPrint this help message.");
    println!("  -v, --version\t\tPrint the version.");
    println!("  -V, --verbose\t\tMore verbose output.");
}

fn version() {
    println!("{} v{}", APP_NAME, APP_VER);
}

// Function to check a dependancy is installed & in PATH
async fn check_dep(dep: &str) -> bool {
    let output = Command::new("which")
        .arg(dep)
        .output()
        .await
        .expect("Failed to execute process");

    if output.status.success() {
        return true;
    } else {
        return false;
    }
}

fn parse_config() -> HashMap<String, String> {
    let config_dir = std::path::Path::new(&*HOME_DIR).join(".config").join("topaz");
    let config_file = config_dir.join("topaz.conf");
    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).expect("Failed to create config directory");
    }
    if !config_file.exists() {
        std::fs::File::create(&config_file).expect("Failed to create config file");
    }

    unsafe {
        STATE.insert("config_dir".to_string(), config_dir.to_str().unwrap().to_string());
        STATE.insert("config_file".to_string(), config_file.to_str().unwrap().to_string());
    }

    let file = std::fs::read_to_string(config_file.to_str().unwrap()).unwrap();
    let mut config = std::collections::HashMap::new();
    for line in file.split("\n") {
        let line = line
            .split("#").next().unwrap()
            .split("=").collect::<Vec<&str>>();
        if line.len() == 2 {
            config.insert(line[0].to_string(), line[1].to_string());
        }
    }
    config
}

async fn run(filepath: &str, _sender: Sender<()>) {
    let child = Command::new("nice")
        .args(&[
            "xwinwrap",
            "-b",
            "-s",
            "-fs",
            "-st",
            "-sp",
            "-nf",
            "-ov",
            "-fdt",
            "--",
            "mpv",
            "-wid %WID",
            "--loop",
            "--no-audio",
            "--panscan=1.0",
            "--framedrop=vo",
            filepath
        ])
        .spawn()
        .expect("Failed to execute process");

    println!("Done");
}

fn kill() {
    // TODO: Gracefully kill any running child xwinwrap / mpv proccesses
    // Kill run() function if running
    
}

#[tokio::main]
async fn main() {
    // Check dependancies
    if !check_dep("xwinwrap").await {
        println!("Error: xwinwrap not found in PATH. Please install it and try again.");
        std::process::exit(1);
    }
    if !check_dep("mpv").await {
        println!("Error: mpv not found in PATH. Please install it and try again.");
        std::process::exit(1);
    }

    let mut verbose_output = true;

    let args = std::env::args().collect::<Vec<String>>();
    for arg in args.iter() {
        if arg == &args[0] {
            continue;
        }
        match arg.as_str() {
            "-h" | "--help" => {
                help();
                std::process::exit(0);
            },
            "-v" | "--version" => {
                version();
                std::process::exit(0);
            },
            "-V" | "--verbose" => {
                verbose_output = true;
            },
            _ => {
                println!("Error: Unknown option: {}", arg);
                std::process::exit(1);
            }
        }
    }

    // Parse config
    let config = parse_config();
    if verbose_output {
        for (key, value) in config.iter() {
            println!("{}: {}", key, value);
        }
    }
    if !config.contains_key("file") {
        println!("Error: No file specified in {}", unsafe { &STATE.get("config_file").unwrap() });
        println!("\nPass '-h' or '--help' for usage.");
        std::process::exit(1);
    }

    // let (send, mut recv) = channel(1);

    println!("Loaded config from {}", unsafe { &STATE.get("config_file").unwrap() });
    println!("Watching for config file changes...");
    let mut hotwatch = Hotwatch::new().expect("Failed to create hotwatch instance");
    hotwatch.watch(unsafe { &STATE.get("config_file").unwrap() }, move |event: Event| {
        match event {
            Event::Create(path) => {
                println!("Config file created: {:?}", path);
                parse_config();
                // TODO: Quit existing xwinwrap process and start a new one
                // Probably make a new function that wraps the xwinwrap command
                // and allows for easy termination + config reload
            },
            Event::Write(path) => {
                println!("Config file modified: {:?}", path);
                parse_config();
                // TODO: ^^
            },
            _ => {
                println!("Unknown event: {:?}", event);
            }
        }
    }).expect("Failed to watch config file");

    if verbose_output {
        println!("Waiting for exit signal...");
    }

    match signal::ctrl_c().await {
        Ok(()) => {},
        Err(err) => {
            eprintln!("Unable to listen for shutdown signal: {}", err);
            // we also shut down in case of error
        },
    }

    if verbose_output {
        println!("\rExiting...");
    }

    // drop(send);
    // let _ = recv.recv().await;
}
