#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod initialize;
mod lyrics;
mod lyrics_window;
mod player;
mod types;
mod ui;

use anyhow::Result;
use log::error;
use log::info;

use initialize::initialize;
use lyrics_window::LyricsWindow;
use ui::run_message_loop;

fn main() -> Result<()> {
    let run = || -> Result<()> {
        initialize()?;
        info!("Initialized");
        let lyrics_window = &mut LyricsWindow::new()?;
        lyrics_window.show()?;
        run_message_loop();
        Ok(())
    };
    let result = run();
    if let Err(e) = result.as_ref() {
        error!("Unexcepted error: {:?}", e);
    }
    result
}
