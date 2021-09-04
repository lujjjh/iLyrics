use std::time::Duration;

use lrc::Lyrics;
use reqwest::blocking::Client;

pub struct Query {
    client: Client,
    last_query: String,
}

impl Query {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            client,
            last_query: String::from(""),
        }
    }

    // TODO: async
    // TODO: refactor
    pub fn get_lyrics(
        &mut self,
        name: &str,
        artist: &str,
        lyrics: &mut Option<Lyrics>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let query = format!("{} {}", name, artist);
        if self.last_query != query {
            self.last_query = query;
            *lyrics = None;
            let body = self
                .client
                .get("https://lyrics-api.lujjjh.com/")
                .query(&[("name", name), ("artist", artist)])
                .send()?
                .text()?;
            *lyrics = Some(lrc::Lyrics::from_str(body)?);
            Ok(true)

            // self.lines = lyrics
            //     .get_timed_lines()
            //     .as_ref()
            //     .iter()
            //     .map(|(time_tag, s)| {
            //         let duration = Duration::from_millis(time_tag.get_timestamp() as u64);
            //         let s = html_escape::decode_html_entities(&s).trim().to_string();
            //         (duration, s)
            //     })
            //     .collect::<Vec<(Duration, String)>>();
        } else {
            Ok(false)
        }
    }
}
