use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::process::Command;
use once_cell::sync::Lazy;
use hotwatch::{Hotwatch, Event};
use tokio::signal::unix::{signal, SignalKind};

const APP_NAME: &str = "Topaz";
const APP_VER: &str = "0.1.0";

static HOME_DIR: Lazy<String> = Lazy::new(|| home::home_dir().unwrap().to_str().unwrap().to_string());

static mut STATE: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut state = HashMap::new();
    state.insert("app_name".to_string(), APP_NAME.to_string());
    state.insert("app_ver".to_string(), APP_VER.to_string());
    state
});

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

async fn spawn_xwinwrap(filepath: &str) -> tokio::process::Child {
    Command::new("nice")
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
        .expect("Failed to execute process")
}

async fn handle_config_change(child_process: Arc<tokio::sync::Mutex<Option<tokio::process::Child>>>, _path: PathBuf) {
    let config = parse_config();
    if let Some(filepath) = config.get("file") {
        let filepath = filepath.clone();

        // Stop the running xwinwrap process
        let mut child_process = child_process.lock().await;
        if let Some(child) = child_process.as_mut() {
            child.kill().await.expect("Failed to kill xwinwrap process");
        }
        *child_process = None;

        // Start a new xwinwrap process with the updated config
        *child_process = Some(spawn_xwinwrap(&filepath).await);
    }
}

#[tokio::main]
async fn main() {
    if !check_dep("xwinwrap").await {
        println!("Error: xwinwrap not found in PATH. Please install it and try again.");
        std::process::exit(1);
    }
    if !check_dep("mpv").await {
        println!("Error: mpv not found in PATH. Please install it and try again.");
        std::process::exit(1);
    }

    let config = parse_config();
    if !config.contains_key("file") {
        println!("Error: No file specified in {}", unsafe { &STATE.get("config_file").unwrap() });
        std::process::exit(1);
    }

    let child_process = Arc::new(tokio::sync::Mutex::new(None));
    {
        let child_process = child_process.clone();
        let filepath = config.get("file").unwrap().clone();
        tokio::spawn(async move {
            let mut child_process = child_process.lock().await;
            *child_process = Some(spawn_xwinwrap(&filepath).await);
        });
    }

    let mut hotwatch = Hotwatch::new().expect("Failed to create hotwatch instance");
    let arc_cloned_child = Arc::clone(&child_process);

    hotwatch.watch(unsafe { &STATE.get("config_file").unwrap() }, move |event: Event| {
        let child_process = arc_cloned_child.clone();
        match event {
            Event::Create(path) | Event::Write(path) => {
                tokio::runtime::Handle::current().block_on(handle_config_change(child_process, path));
            },
            _ => {
                println!("Unknown event: {:?}", event);
            }
        }
    }).expect("Failed to watch config file");

    // Set up signal handlers for SIGTERM and SIGINT
    let mut signal_handler = signal(SignalKind::terminate()).expect("Failed to create SIGTERM handler");
    let mut ctrl_c = signal(SignalKind::interrupt()).expect("Failed to create SIGINT handler");
    
    tokio::select! {
        _ = ctrl_c.recv() => {
            println!("SIGINT received, exiting...");
        }
        _ = signal_handler.recv() => {
            println!("SIGTERM received, exiting...");
        }
    }
    
    // Kill the xwinwrap process before exiting
    let mut child_process = child_process.lock().await;
    if let Some(child) = child_process.as_mut() {
        child.kill().await.expect("Failed to kill xwinwrap process");
    }
    *child_process = None;
}
