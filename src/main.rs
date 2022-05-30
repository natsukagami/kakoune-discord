use discord_sdk::Discord;
use log::info;
use simple_logger::SimpleLogger;
use std::{env, fs, time};

const CLIENT_ID: i64 = 561241836451004449;

#[tokio::main]
async fn main() {
    // Set up logger
    SimpleLogger::new().init().unwrap();

    let filename = env::args()
        .nth(1)
        .expect("Expected a filename to read from.");
    let mut kak_count = 0;

    // Start a discord client
    let client = Discord::new(
        CLIENT_ID,
        discord_sdk::Subscriptions::ACTIVITY,
        Box::new(discord_sdk::handlers::Printer),
    )
    .expect("I should have a client");

    loop {
        let info_bytes = fs::read(&filename).expect("Something went wrong with reading the fifo");
        let info_raw =
            String::from_utf8(info_bytes).expect("Something went wrong with parsing the bytes");
        let info = info_raw.trim_end();

        info!("Received from kak: {}", info);

        if info == "+" {
            kak_count += 1;
        } else if info == "-" {
            kak_count -= 1;
            if kak_count == 0 {
                fs::remove_file(filename).expect("Something went wrong with removing the fifo");
                break;
            }
        } else {
            let now = time::SystemTime::now();
            let epoc_secs = now
                .duration_since(time::UNIX_EPOCH)
                .expect("Epoch is after now?")
                .as_secs() as i64;
            client
                .update_activity(
                    discord_sdk::activity::ActivityBuilder::new()
                        .details(format!("Editing {}", info.replace("'", "")))
                        .start_timestamp(epoc_secs)
                        .assets(
                            discord_sdk::activity::Assets::default()
                                .large("default", None as Option<String>),
                        ),
                )
                .await
                .expect("Failed to set activity");
        }
    }
}
