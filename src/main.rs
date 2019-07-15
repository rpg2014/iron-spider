extern crate config;
extern crate discord;
extern crate pretty_env_logger;
extern crate reqwest;
#[macro_use]
extern crate log;

use std::env;
use std::error::Error;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
mod discord_listener;
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn Error>> {
    pretty_env_logger::init();
    let (tx, rx) = channel();
    let mut settings = config::Config::default();
    if bool::from_str(&env::var("ON_CLOUD").unwrap_or("false".to_string()))? {
        //if in cloud or docker container
        //settings.set("ON_CLOUD",true).unwrap();
        settings
            .set(
                "URL",
                env::var("URL").unwrap_or("https://jsonplaceholder.typicode.com/posts".to_string()),
            )
            .unwrap();
        settings
            .merge(config::Environment::with_prefix("IRON_SPIDER"))
            .unwrap();
        settings.set("ON_CLOUD", true).unwrap();
        settings.set("PORT", env::var("PORT").unwrap()).unwrap();
        info!("On docker")
    } else {
        info!("Running locally");
        settings
            .merge(config::File::with_name(".settings.yaml"))
            .unwrap();
        settings.set("ON_CLOUD", false).unwrap();
    }

    //will listen so we dont get killed
    let port = settings.get_str("PORT").unwrap();
    let redirect_url = settings.get_str("S3_URL")?;
    thread::spawn(move || {
        start_redirect_server(&port, &redirect_url);
    });

    //cloning url here before the settings are moved into the run_discord function
    let url: String = settings.get("URL").unwrap();
    thread::spawn(move || {
        discord_listener::run_discord(tx, settings);
    });

    let client = reqwest::Client::new();

    loop {
        let text: String = rx.recv().unwrap_or_default();
        if text.chars().count() >= 2 {
            print!("Sending text \"{}\" to url: {}\t->", text, url);
            let mut map = HashMap::new();
            map.insert("text", text);
            let res = client
                .post(&url)
                .json(&map)
                .send()
                .expect("URL failed to parse");
            info!("\tResponse: {}", res.status());
        }
    }
}

fn start_redirect_server(port: &String, redirect_url: &String) {
    let mut redirect_response = String::from("HTTP/1.1 301 Moved Permanently\nLocation: ");
    redirect_response.push_str(redirect_url);
    let mut ip = "0.0.0.0:".to_owned();
    ip.push_str(&port);
    info!("Binding to {}", ip);
    let listener = TcpListener::bind(ip).unwrap();
    for stream in listener.incoming() {
        info!("Got tcp request");
        if let Ok(mut s) = stream {
            let mut data = [0 as u8; 200]; // using 200 byte buffer
                                           // this next block is the conditional of the while block
            while match s.read(&mut data) {
                Ok(_) => {
                    info!(
                        "Data Received: {}",
                        String::from_utf8(data.to_vec()).unwrap()
                    );
                    // return Redirect response
                    s.write(redirect_response.as_bytes()).unwrap();
                    false
                }
                Err(_) => false,
            } {} // this thing is the body of the while loop
        }
    }
}
