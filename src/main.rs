mod audio;

use audio::{BandEnergy, SharedAudio};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use image::{imageops::FilterType, GrayImage};
use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
    Camera,
};
use std::{
    io::{self, Write},
    sync::{Arc, Mutex},
    time::Duration,
};

// ── Schriftsysteme ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq)]
pub enum Script {
    Latin,
    Cyrillic,
    Hiragana,
    Katakana,
    Arabic,
    Braille,
}

impl Script {
    const ALL: &'static [Script] = &[
        Script::Latin, Script::Cyrillic, Script::Hiragana,
        Script::Katakana, Script::Arabic, Script::Braille,
    ];

    fn name(self) -> &'static str {
        match self {
            Script::Latin    => "Latin",
            Script::Cyrillic => "Кирилл",
            Script::Hiragana => "ひらがな",
            Script::Katakana => "カタカナ",
            Script::Arabic   => "عربي",
            Script::Braille  => "Braille",
        }
    }

    fn chars(self) -> &'static [char] {
        match self {
            Script::Latin => &[
                ' ', '.', '\'', '`', '^', ',', ':', ';', 'I', 'l', '!', 'i', '>',
                '<', '+', '_', '-', '?', ']', '[', '1', ')', '(', '|', 't', 'f',
                'r', 'x', 'n', 'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L',
                'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k', 'h', 'a',
                'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$',
            ],
            Script::Cyrillic => &[
                ' ', '·', 'і', 'ї', 'є', 'а', 'е', 'о', 'с', 'х', 'р', 'н', 'к',
                'з', 'и', 'т', 'г', 'д', 'у', 'ф', 'б', 'в', 'й', 'л', 'м', 'п',
                'ц', 'ч', 'ш', 'э', 'ю', 'я', 'Д', 'Ж', 'З', 'И', 'Й', 'Л', 'П',
                'Ф', 'Ц', 'Ч', 'Ш', 'Щ', 'Э', 'Ю', 'Я', 'Б', 'В', 'Г', 'Е', 'М',
                'Н', 'Т', 'Х', 'Ъ', 'Ы', 'Ь', 'А', 'О', 'С', 'К', 'Р', 'У',
            ],
            Script::Hiragana => &[
                ' ', 'あ', 'い', 'う', 'え', 'お', 'か', 'き', 'く', 'け', 'こ',
                'さ', 'し', 'す', 'せ', 'そ', 'た', 'ち', 'つ', 'て', 'と', 'な',
                'に', 'ぬ', 'ね', 'の', 'は', 'ひ', 'ふ', 'へ', 'ほ', 'ま', 'み',
                'む', 'め', 'も', 'や', 'ゆ', 'よ', 'ら', 'り', 'る', 'れ', 'ろ',
                'わ', 'を', 'ん', 'が', 'ぎ', 'ぐ', 'げ', 'ご', 'ざ', 'じ', 'ず',
                'ぜ', 'ぞ', 'だ', 'ぢ', 'づ', 'で', 'ど',
            ],
            Script::Katakana => &[
                ' ', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ',
                'サ', 'シ', 'ス', 'セ', 'ソ', 'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ',
                'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ', 'ミ',
                'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル', 'レ', 'ロ',
                'ワ', 'ヲ', 'ン', 'ガ', 'ギ', 'グ', 'ゲ', 'ゴ', 'ザ', 'ジ', 'ズ',
                'ゼ', 'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド',
            ],
            Script::Arabic => &[
                ' ', '·', 'ء', 'آ', 'أ', 'إ', 'ا', 'ب', 'ت', 'ث', 'ج', 'ح', 'خ',
                'د', 'ذ', 'ر', 'ز', 'س', 'ش', 'ص', 'ض', 'ط', 'ظ', 'ع', 'غ', 'ف',
                'ق', 'ك', 'ل', 'م', 'ن', 'ه', 'و', 'ي', 'ى', 'ة', 'ئ', 'ؤ',
            ],
            Script::Braille => &[
                ' ', '⠁', '⠂', '⠃', '⠄', '⠅', '⠆', '⠇', '⠈', '⠉', '⠊', '⠋',
                '⠌', '⠍', '⠎', '⠏', '⠐', '⠑', '⠒', '⠓', '⠔', '⠕', '⠖', '⠗',
                '⠘', '⠙', '⠚', '⠛', '⠜', '⠝', '⠞', '⠟', '⠠', '⠡', '⠢', '⠣',
                '⠤', '⠥', '⠦', '⠧', '⠨', '⠩', '⠪', '⠫', '⠬', '⠭', '⠮', '⠯',
                '⠰', '⠱', '⠲', '⠳', '⠴', '⠵', '⠶', '⠷', '⠸', '⠹', '⠺', '⠻',
                '⠼', '⠽', '⠾', '⠿',
            ],
        }
    }

    fn next(self) -> Script {
        let idx = Script::ALL.iter().position(|&s| s == self).unwrap_or(0);
        Script::ALL[(idx + 1) % Script::ALL.len()]
    }
}

// ── Band→Parameter Mapping ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BandTarget {
    None,
    Contrast,
    Resolution,
    Script,
    Density,
}

impl BandTarget {
    fn name(self) -> &'static str {
        match self {
            BandTarget::None       => "—",
            BandTarget::Contrast   => "Kontrast",
            BandTarget::Resolution => "Auflösung",
            BandTarget::Script     => "Schrift",
            BandTarget::Density    => "Dichte",
        }
    }

    fn cycle(self) -> BandTarget {
        use BandTarget::*;
        match self { None => Contrast, Contrast => Resolution,
                     Resolution => Script, Script => Density, Density => None }
    }
}

// ── App State ─────────────────────────────────────────────────────────────────

struct State {
    script: Script,
    density: f32,      // 0.3–1.0: fraction of charset used
    inverted: bool,
    contrast: f32,     // 0.5–4.0
    scale: u32,        // terminal cols / scale = ascii cols

    mappings: [BandTarget; 3],
    active_band: usize,
}

impl Default for State {
    fn default() -> Self {
        State {
            script: Script::Latin,
            density: 1.0,
            inverted: false,
            contrast: 1.0,
            scale: 2,
            mappings: [BandTarget::Contrast, BandTarget::Resolution, BandTarget::Script],
            active_band: 0,
        }
    }
}

impl State {
    fn charset(&self) -> &'static [char] {
        let all = self.script.chars();
        let take = ((all.len() as f32 * self.density) as usize).max(2).min(all.len());
        &all[..take]
    }

    fn apply_audio(&mut self, band: &BandEnergy) {
        let energies = [band.bass, band.mid, band.high];
        for (i, &e) in energies.iter().enumerate() {
            match self.mappings[i] {
                BandTarget::Contrast   => self.contrast = 0.5 + e * 3.5,
                BandTarget::Resolution => self.scale = (1.0 + e * 5.0).round() as u32,
                BandTarget::Density    => self.density = (0.3 + e * 0.7).clamp(0.0, 1.0),
                BandTarget::Script | BandTarget::None => {}
            }
        }
    }
}

// ── ASCII conversion ──────────────────────────────────────────────────────────

fn frame_to_ascii(gray: &GrayImage, width: u32, height: u32, state: &State) -> Vec<String> {
    let charset = state.charset();
    let len = charset.len() as f32;
    let resized = image::imageops::resize(gray, width, height, FilterType::Nearest);

    resized.rows().map(|row| {
        row.map(|px| {
            let mut luma = px.0[0] as f32 / 255.0;
            luma = ((luma - 0.5) * state.contrast + 0.5).clamp(0.0, 1.0);
            if state.inverted { luma = 1.0 - luma; }
            let idx = ((luma * (len - 1.0)).round() as usize).min(charset.len() - 1);
            charset[idx]
        }).collect()
    }).collect()
}

fn vu_bar(energy: f32) -> String {
    let n = (energy * 8.0).round() as usize;
    format!("{}{}", "█".repeat(n), "░".repeat(8usize.saturating_sub(n)))
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> anyhow::Result<()> {
    // Audio setup
    let shared_audio: SharedAudio = Arc::new(Mutex::new(BandEnergy::default()));
    let mut _stream = None;
    if let Some(dev) = audio::default_device() {
        eprintln!("Audio: {}", dev.name);
        _stream = Some(audio::start_capture(dev, shared_audio.clone())?);
    } else {
        eprintln!("Kein Audio-Gerät gefunden — laufe ohne Audio");
    }

    // Webcam
    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
    let mut camera = Camera::new(CameraIndex::Index(0), format)?;
    camera.open_stream()?;

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    let mut state = State::default();
    let mut last_script_energy = 0.0f32;
    let mut running = true;

    while running {
        // ── Input ─────────────────────────────────────────────────────────
        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => running = false,
                    KeyCode::Char('i') => state.inverted = !state.inverted,
                    KeyCode::Char('1') => state.active_band = 0,
                    KeyCode::Char('2') => state.active_band = 1,
                    KeyCode::Char('3') => state.active_band = 2,
                    KeyCode::Char('m') => {
                        state.mappings[state.active_band] =
                            state.mappings[state.active_band].cycle();
                    }
                    // Manuelle Overrides
                    KeyCode::Char('+') | KeyCode::Char('=') =>
                        state.scale = (state.scale + 1).min(8),
                    KeyCode::Char('-') =>
                        state.scale = state.scale.saturating_sub(1).max(1),
                    KeyCode::Char('s') => state.script = state.script.next(),
                    _ => {}
                }
            }
        }

        // ── Audio ──────────────────────────────────────────────────────────
        let band = shared_audio.lock().map(|g| g.clone()).unwrap_or_default();
        state.apply_audio(&band);

        // Script trigger: steigende Flanke über 0.75
        let script_energy = [band.bass, band.mid, band.high]
            .iter()
            .zip(state.mappings.iter())
            .filter_map(|(&e, &t)| if t == BandTarget::Script { Some(e) } else { None })
            .fold(0.0f32, f32::max);

        if script_energy > 0.75 && last_script_energy <= 0.75 {
            state.script = state.script.next();
        }
        last_script_energy = script_energy;

        // ── Kamera ─────────────────────────────────────────────────────────
        let frame = match camera.frame() { Ok(f) => f, Err(_) => continue };
        let decoded = frame.decode_image::<RgbFormat>()?;
        let rgb = image::RgbImage::from_raw(decoded.width(), decoded.height(), decoded.into_raw())
            .expect("buffer mismatch");
        let gray = image::DynamicImage::ImageRgb8(rgb).to_luma8();

        let (term_cols, term_rows) = terminal::size()?;
        let scale = state.scale.max(1);
        let ascii_cols = (term_cols as u32 / scale).max(1);
        let ascii_rows = (term_rows.saturating_sub(3) as u32 / scale / 2).max(1);

        let lines = frame_to_ascii(&gray, ascii_cols, ascii_rows, &state);

        // ── Render ─────────────────────────────────────────────────────────
        queue!(stdout, cursor::MoveTo(0, 0), terminal::Clear(ClearType::All))?;
        for line in &lines {
            queue!(stdout, Print(line), Print("\r\n"))?;
        }

        // Band-Anzeige (2. letzte Zeile)
        let band_data = [
            ("Bass", band.bass, Color::Red),
            ("Mid ", band.mid,  Color::Green),
            ("High", band.high, Color::Cyan),
        ];
        queue!(stdout, cursor::MoveTo(0, term_rows - 3))?;
        for (i, (label, energy, color)) in band_data.iter().enumerate() {
            let sel = if i == state.active_band { '▶' } else { ' ' };
            let target = state.mappings[i].name();
            queue!(
                stdout,
                SetForegroundColor(*color),
                Print(format!("{sel}{label}→{:<9} {} ", target, vu_bar(*energy))),
                ResetColor,
            )?;
        }

        // Status (letzte Zeile)
        let status = format!(
            " {script} | kontrast:{:.1} skala:{scale} dichte:{:.0}% | \
             1/2/3:band  m:remap  i:invert  s:script  q:quit",
            state.contrast,
            state.density * 100.0,
            script = state.script.name(),
        );
        let truncated: String = status.chars().take(term_cols as usize).collect();
        queue!(stdout, cursor::MoveTo(0, term_rows - 1), Print(truncated))?;

        stdout.flush()?;
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;
    camera.stop_stream()?;
    println!("Tschüss!");
    Ok(())
}
