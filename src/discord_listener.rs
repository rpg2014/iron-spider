use std::sync::mpsc::Sender;
use discord::model::Event;
use discord::Discord;
use std::process;


//move below to new file
pub fn run_discord(tx: Sender<String>, settings: config::Config) {
	let discord;
	if settings.get("ON_CLOUD").unwrap() {
		discord = Discord::from_bot_token(&settings.get_str("IRON_SPIDER_DISCORD_TOKEN").unwrap_or("None".to_string()))
			.expect("login failed");
	} else {
		discord = get_local_discord(&settings)
	}

	let (mut connection, _) = discord.connect().expect("connect failed");
	println!("Gateway Connected, listening...");
	loop {
		let event = match connection.recv_event() {
			Ok(event) => event,
			Err(err) => {
				println!("Receive error: {:?}", err);
				if let discord::Error::WebSocket(..) = err {
						//Handle dropped web connection
						let (new_conn,_) = discord.connect().expect("connection failed");
						connection = new_conn;
						println!("Reconnected");
					}
				if let discord::Error::Closed(..) = err {
					break
				}
				continue
				},
		};

		match event {
			Event::MessageCreate(message) => {
				let text: String = message.content.clone();
				if text.starts_with("!") {
					match text.as_ref() {
						//add so it only checks first word, and !server  is only passed through
						"!quit" => {
							println!("Recived exit command");
							process::exit(1);
						}
						_ => {
							let text_clone = text.clone();
							//refactor below (move to new method)
							match tx.send(text) {
								Ok(_) => {
									if settings.get("dev").unwrap_or(false) {
										// running locally
										let log = format!("Sent through channel: {} ", text_clone);
										let _ = discord.send_message(
											message.channel_id,
											&log,
											"",
											false,
										);
									}
								}
								Err(error) => println!("Error: channel disconnected: {}", error),
							}
						}
					}
				} else if message.content == "!quit" {
					println!("Quitting.");
					break;
				}
			}
			_ => {}
		}
	}
}

fn get_local_discord(settings: &config::Config) -> Discord {
	let token: String = settings.get(&"bot_token").unwrap();
	return Discord::from_bot_token(&token).expect("login failed");
}
