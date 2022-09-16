use tokio::process::Command;

#[tokio::main]
async fn main() {
    // println!("Hello, world!");

    Command::new("echo")
        .arg("Hello, world!")
        .spawn()
        .expect("Hello, world!");
}
