extern crate config;
extern crate discord;
extern crate pretty_env_logger;
extern crate reqwest;
#[macro_use]
extern crate log;

use std::env;
use std::error::Error;
mod http_redirect_server;
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
        settings.set(
            "URL",
            env::var("URL").unwrap_or("https://jsonplaceholder.typicode.com/posts".to_string()),
        )?;
        settings.merge(config::Environment::with_prefix("IRON_SPIDER"))?;
        settings.set("ON_CLOUD", true)?;
        settings.set("PORT", env::var("PORT")?)?;
        info!("On docker")
    } else {
        info!("Running locally");
        settings.merge(config::File::with_name(".settings.yaml"))?;
        settings.set("ON_CLOUD", false)?;
    }

    //spawn redirect server
    let port = settings.get_str("PORT")?;
    let redirect_url = settings.get_str("S3_URL")?;
    thread::spawn(move || while match http_redirect_server::start_redirect_server(&port, &redirect_url) {
        Ok(_) => true,
        Err(e) => {println!("{:?}", e);
                    true    
                  },
    }{});

    //cloning url here before the settings are moved into the run_discord function
    let url: String = settings.get("URL")?;
    thread::spawn(move || {
        discord_listener::run_discord(tx, settings);
    });

    let client = reqwest::Client::new();

//TODO: make it so an auth token is sent along with the text
    loop {
        let text: String = rx.recv().unwrap_or_default();
        if text.chars().count() >= 2 {
            info!("Sending text \"{}\" to url: {}\t->", text, url);
            let mut map = HashMap::new();
            map.insert("text", text);
            //map.insert("auth_token", authToken);
            let res = client
                .post(&url)
                .json(&map)
                .send()
                .expect("URL failed to parse");
            info!("\tResponse: {}", res.status());
        }
    }
}


