use ratatui::style::Color;
use std::time::Instant;

const FADE_MS: f32 = 300.0;
const FLASH_MS: f32 = 400.0;

pub struct AnimationState {
    fade_start: Option<Instant>,
    flash: Option<(usize, Instant)>,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            fade_start: None,
            flash: None,
        }
    }

    pub fn is_animating(&self) -> bool {
        self.fade_start.is_some() || self.flash.is_some()
    }

    pub fn start_fade(&mut self) {
        self.fade_start = Some(Instant::now());
    }

    pub fn fade_progress(&self) -> f32 {
        if let Some(start) = self.fade_start {
            ease_out_cubic(progress(start, FADE_MS))
        } else {
            1.0
        }
    }

    pub fn start_flash(&mut self, verse_idx: usize) {
        self.flash = Some((verse_idx, Instant::now()));
    }

    pub fn flash_intensity(&self, verse_idx: usize) -> f32 {
        if let Some((idx, start)) = self.flash {
            if idx == verse_idx {
                let t = progress(start, FLASH_MS);
                if t < 0.2 {
                    t / 0.2
                } else if t < 1.0 {
                    1.0 - (t - 0.2) / 0.8
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    pub fn tick(&mut self) {
        if let Some(start) = self.fade_start {
            if progress(start, FADE_MS) >= 1.0 {
                self.fade_start = None;
            }
        }
        if let Some((_, start)) = self.flash {
            if progress(start, FLASH_MS) >= 1.0 {
                self.flash = None;
            }
        }
    }
}

fn progress(start: Instant, duration_ms: f32) -> f32 {
    let elapsed = start.elapsed().as_secs_f32() * 1000.0;
    (elapsed / duration_ms).min(1.0)
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    if let (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) = (a, b) {
        let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Color::Rgb(lerp(r1, r2), lerp(g1, g2), lerp(b1, b2))
    } else {
        if t < 0.5 { a } else { b }
    }
}
