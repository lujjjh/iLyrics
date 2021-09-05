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
            let downloaded_lyrics = Lyrics::from_str(body)?;
            let mut new_lyrics = Lyrics::new();
            let timed_lines = downloaded_lyrics.get_timed_lines();
            for (i, (time_tag, line)) in timed_lines.iter().enumerate() {
                let line = html_escape::decode_html_entities(&line);
                let text = line.trim();
                // Skip empty lines that last no longer than 3s.
                if text.is_empty() {
                    if i < timed_lines.len() - 1 {
                        let duration =
                            timed_lines[i + 1].0.get_timestamp() - time_tag.get_timestamp();
                        if duration <= 3000 {
                            continue;
                        }
                    }
                }
                new_lyrics.add_timed_line(time_tag.clone(), line)?;
            }
            *lyrics = Some(new_lyrics);
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
