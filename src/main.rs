use discord_sdk::Discord;
use log::{error, info};
use simple_logger::SimpleLogger;
use std::{env, time};
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

const CLIENT_ID: i64 = 561241836451004449;

type F = BufReader<File>;
struct FifoReader {
    filename: String,
    lines: tokio::io::Lines<F>,
}

impl FifoReader {
    async fn lines(filename: &str) -> tokio::io::Lines<F> {
        let file = File::open(filename)
            .await
            .expect("Should be able to open fifo file");
        let file = BufReader::new(file);
        file.lines()
    }

    pub async fn new(filename: String) -> FifoReader {
        let lines = Self::lines(&filename[..]).await;
        Self { filename, lines }
    }

    #[async_recursion::async_recursion]
    pub async fn next(&mut self) -> String {
        match self.lines.next_line().await.expect("Cannot read line") {
            Some(v) => v,
            None => {
                self.lines = Self::lines(self.filename.as_ref()).await;
                self.next().await
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Set up logger
    SimpleLogger::new().init().unwrap();

    let filename = env::args()
        .nth(1)
        .expect("Expected a filename to read from.");

    let mut file = FifoReader::new(filename.clone()).await;

    let mut kak_count = 0;

    // Start a discord client
    let (wheel, handler) = discord_sdk::wheel::Wheel::new(Box::new(|err| {
        error!("encountered an error: {}", err);
    }));

    let mut user = wheel.user();
    let client = Discord::new(
        CLIENT_ID,
        discord_sdk::Subscriptions::ACTIVITY,
        Box::new(handler),
    )
    .expect("I should have a client");
    info!("waiting for handshake...");
    user.0.changed().await.unwrap();

    let user = match &*user.0.borrow() {
        discord_sdk::wheel::UserState::Connected(user) => user.clone(),
        discord_sdk::wheel::UserState::Disconnected(err) => {
            panic!("failed to connect to Discord: {}", err)
        }
    };

    info!("connected to Discord, local user is {:#?}", user);

    loop {
        let info = file.next().await;
        let info = info.trim_end().to_owned();

        info!("Received from kak: {}", info);

        if info == "+" {
            kak_count += 1;
        } else if info == "-" {
            kak_count -= 1;
            if kak_count == 0 {
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
                                .large("default", Option::<String>::None),
                        ),
                )
                .await
                .expect("Failed to set activity");
        }
    }
    std::fs::remove_file(filename).expect("Something went wrong with removing the fifo");
}
