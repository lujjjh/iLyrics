use std::time::Duration;

use reqwest::Client;

pub struct Lyrics {
    client: Client,
    last_query: String,
    lyrics: Option<lrc::Lyrics>,
    lines: Vec<(Duration, String)>,
}

impl Lyrics {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap();
        Self {
            client,
            last_query: String::from(""),
            lyrics: None,
            lines: vec![],
        }
    }

    pub async fn get_lyrics_line(
        &mut self,
        name: &str,
        artist: &str,
        duration: Duration,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let query = format!("{} {}", name, artist);
        if self.last_query != query {
            self.last_query = query;
            self.lyrics = None;
            let body = self
                .client
                .get("https://lyrics-api.lujjjh.com/")
                .query(&[("name", name), ("artist", artist)])
                .send()
                .await?
                .text()
                .await?;
            let lyrics = lrc::Lyrics::from_str(body)?;
            self.lines = lyrics
                .get_timed_lines()
                .as_ref()
                .iter()
                .map(|(time_tag, s)| {
                    let duration = Duration::from_millis(time_tag.get_timestamp() as u64);
                    let s = html_escape::decode_html_entities(&s).trim().to_string();
                    (duration, s)
                })
                .collect::<Vec<(Duration, String)>>();
            self.lyrics = Some(lyrics);
        }
        Ok(if let Some(lyrics) = &self.lyrics {
            lyrics
                .find_timed_line_index(duration.as_millis() as i64)
                .map(|index| self.lines[index].1.clone())
        } else {
            None
        })
    }
}

#[tokio::test]
async fn test_get_lyrics() {
    let mut lyrics = Lyrics::new();
    println!(
        "{:?}",
        lyrics
            .get_lyrics_line("Lemon Tree", "Fool's Garden", Duration::from_secs(30))
            .await
            .unwrap()
    );
    println!(
        "{:?}",
        lyrics
            .get_lyrics_line("Lemon Tree", "Fool's Garden", Duration::from_secs(40))
            .await
            .unwrap()
    );
}
