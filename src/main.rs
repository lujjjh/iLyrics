#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod initialize;
mod lyrics;
mod lyrics_window;
mod player;
mod types;
mod ui;

use windows::*;

use initialize::initialize;
use lyrics_window::LyricsWindow;
use ui::run_message_loop;

fn main() -> Result<()> {
    initialize()?;
    let lyrics_window = &mut LyricsWindow::new()?;
    lyrics_window.show()?;
    run_message_loop();
    Ok(())
}
