extern crate discord;
extern crate config;
extern crate reqwest;

use discord::Discord;
use discord::model::Event;
use std::env;
use std::str::FromStr;
use std::sync::mpsc::{Sender, channel};
use std::thread;
use std::process;


fn main() {
	let (tx, rx) = channel();
	let mut settings = config::Config::default();
	settings.merge(config::File::with_name(".settings.yaml")).unwrap();
	let settings_clone = settings.clone();
	thread::spawn(move || {
							run_discord(tx,settings_clone);
						});

	let url : String = settings.get("URL").unwrap();
	let client = reqwest::Client::new();
	loop {
		let text: String = rx.recv().unwrap_or_default();
		if !text.is_empty(){
			print!("Sending text \"{}\" to url: {}\t->",text,url);
			let params = [("text",text)];
			let res = client.post(&url)
    					.form(&params)
    					.send().expect("URL failed to parse");
			println!("\tResponse: {}", res.status());
		}
	}
}


fn run_discord(tx : Sender<String>, mut settings : config::Config) {
	// Log in to Discord using a bot token from the environment
    let discord;
    

    if bool::from_str(&env::var("ON_CLOUD").unwrap_or("false".to_string())).unwrap_or(false) {
        discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Expected token")).expect("login failed");
        settings.merge(config::Environment::with_prefix("APP")).unwrap();
    } else {
        
        discord = get_local_discord(&settings)
    }
   
	 //TODO read from file to get discord bot tokeng
    
	// Establish and use a websocket connection
	
	let (mut connection, _) = discord.connect().expect("connect failed");
	println!("Gateway Connected, listening...");
	loop {
		match connection.recv_event() {
			Ok(Event::MessageCreate(message)) => {
				let text : String = message.content.clone();
				if text.starts_with("!") {
					match text.as_ref() {
						//add so it only checks first work, and !server  is only passed through
						"!quit" => {
							println!("Recived exit command");
							process::exit(1);
							}
						_ => {
							let text_clone = text.clone();
							match  tx.send(text){
								Ok(_) => {
										if settings.get("dev").unwrap_or(false) { // running locally
											let log = format!("Sent through channel: {} ",text_clone);
											let _ = discord.send_message(message.channel_id, &log, "", false);
										}
									
									}
								Err(error) => println!("Error: channel disconnected: {}", error)
							}
						
						}
					
					}
					
					
				} else if message.content == "!quit" {
					println!("Quitting.");
					break
				}
			}
			Ok(_) => {}
			Err(discord::Error::Closed(code, body)) => {
				println!("Gateway closed on us with code {:?}: {}", code, body);
				break
			}
			Err(err) => println!("Receive error: {:?}", err)
		}
	}
}

fn get_local_discord(settings: &config::Config) -> Discord {
    let token : String = settings.get(&"bot_token").unwrap();
    return Discord::from_bot_token(&token).expect("login failed")
}
