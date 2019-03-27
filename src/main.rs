extern crate config;
extern crate discord;
extern crate reqwest;

use std::env;
use std::net::TcpListener;
use std::str::FromStr;
use std::sync::mpsc::channel;
use std::thread;
mod discord_listener;
use std::collections::HashMap;

fn main() {
	let (tx, rx) = channel();
	let mut settings = config::Config::default();
	if bool::from_str(&env::var("ON_CLOUD").unwrap_or("false".to_string())).unwrap() {
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
		println!("On docker")
	} else {
		println!("Running locally");
		settings
			.merge(config::File::with_name(".settings.yaml"))
			.unwrap();
		settings.set("ON_CLOUD", false).unwrap();
	}
	//will listen so we dont get killed
	let port = settings.get_str("PORT").unwrap();
	thread::spawn(move || {
		let mut ip = "0.0.0.0:".to_owned();
		ip.push_str(&port);
		println!("Binding to {}", ip);
		let listener = TcpListener::bind(ip).unwrap();
		for stream in listener.incoming() {
			match stream {
				Ok(_) => {}
				Err(_)=> {}
			}
		}
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
			println!("\tResponse: {}", res.status());
		}
	}
}
