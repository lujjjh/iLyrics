use std::time::Duration;

pub mod itunes;

pub trait Player {
    fn get_player_state(&mut self) -> Option<PlayerState>;
}

#[derive(Debug)]
pub struct PlayerState {
    pub song_name: String,
    pub song_artist: String,
    pub player_position: Duration,
}
