extern crate discord;
extern crate config;

use discord::Discord;
use discord::model::Event;
use std::env;
use std::str::FromStr;

fn main() {
    println!("Hello");
	// Log in to Discord using a bot token from the environment
    let discord;
    let mut settings = config::Config::default();

    if bool::from_str(&env::var("ON_CLOUD").unwrap_or("false".to_string())).unwrap_or(false) {
        discord = Discord::from_bot_token(&env::var("DISCORD_TOKEN").expect("Expected token")).expect("login failed");
        settings.merge(config::Environment::with_prefix("APP")).unwrap();
    } else {
        settings.merge(config::File::with_name(".settings.yaml")).unwrap();
        discord = get_local_discord(&settings)
    }
   
	 //TODO read from file to get discord bot tokeng
    

	// Establish and use a websocket connection
	let (mut connection, _) = discord.connect().expect("connect failed");
	println!("Ready.");
	loop {
		match connection.recv_event() {
			Ok(Event::MessageCreate(message)) => {
				println!("{} says: {}", message.author.name, message.content);
				if message.content == "!test" {
					let _ = discord.send_message(message.channel_id, "This is a reply to the test.", "", false);
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

