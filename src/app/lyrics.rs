use std::time::Duration;

use reqwest::blocking::Client;

pub struct Lyrics {
    client: Client,
    last_query: String,
    lyrics: Option<lrc::Lyrics>,
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
        }
    }

    pub fn get_lyrics_line(
        &mut self,
        q: &str,
        duration: Duration,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        if self.last_query != q {
            self.last_query = q.to_string();
            self.lyrics = None;
            let body = self
                .client
                .get("https://lyrics.lujjjh.workers.dev/")
                .query(&[("q", q)])
                .send()?
                .text()?;
            self.lyrics = Some(lrc::Lyrics::from_str(body)?);
        }
        Ok(if let Some(lyrics) = &self.lyrics {
            lyrics
                .find_timed_line_index(duration.as_millis() as i64)
                .map(|index| {
                    let timed_lines = lyrics.get_timed_lines();
                    timed_lines[index].1.to_string()
                })
        } else {
            None
        })
    }
}

#[test]
fn test_get_lyrics() {
    let mut lyrics = Lyrics::new();
    println!(
        "{:?}",
        lyrics
            .get_lyrics_line("Lemon Tree Fools Garden", Duration::from_secs(30))
            .unwrap()
    );
    println!(
        "{:?}",
        lyrics
            .get_lyrics_line("Lemon Tree Fools Garden", Duration::from_secs(40))
            .unwrap()
    );
}
