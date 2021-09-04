use std::time::Duration;

use bindings::Windows::Win32::Graphics::Direct2D::*;
use lrc::Lyrics;
use windows::*;

use crate::lyrics_window::element::Rectangle;
use crate::types::Rect;

use super::element::Render;

pub(crate) struct LyricsWindowState<'a> {
    lyrics: Option<&'a Lyrics>,
    position: Option<Duration>,
    elements: Vec<Box<dyn Render>>,
    invalidated: bool,
}

impl<'a> LyricsWindowState<'a> {
    pub(crate) fn new() -> Self {
        Self {
            lyrics: None,
            position: None,
            elements: vec![],
            invalidated: true,
        }
    }

    pub(crate) fn set_lyrics(&mut self, lyrics: Option<&'a Lyrics>) {
        self.lyrics = lyrics;
        self.position = None;
        self.invalidated = true;
    }

    pub(crate) fn set_position(&mut self, position: Option<Duration>) {
        if self.position != position {
            self.position = position;
            self.invalidated = true;
        }
    }

    pub(crate) fn get_elements(&mut self) -> &mut [Box<dyn Render>] {
        if self.invalidated {
            let mut rect = Box::new(Rectangle::new(Rect {
                left: 0.,
                top: 0.,
                right: 100.,
                bottom: 100.,
            }));
            rect.set_fill(Some(D2D1_COLOR_F {
                r: 0.,
                g: 0.,
                b: 0.,
                a: 0.5,
            }));
            self.elements = vec![rect];
            self.invalidated = false;
        }
        &mut self.elements
    }
}

impl Render for LyricsWindowState<'_> {
    fn render(&mut self, dc: &ID2D1DeviceContext) -> Result<()> {
        for element in self.get_elements().iter_mut() {
            element.render(dc)?;
        }
        Ok(())
    }
}
