/// Alle Zeichensätze sind nach visueller Deckkraft sortiert:
/// Index 0 = leerster / hellster Charakter, letzter Index = dichtester / dunkelster.
/// Die Kontrast-Option streicht Zeichen von beiden Enden, um den nutzbaren
/// Dynamikbereich einzuengen.

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DetailLevel {
    Fine,
    Medium,
    Coarse,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Charset {
    // Schriftsysteme
    Latin,
    Cyrillic,
    Hiragana,
    Katakana,
    Arabic,
    Braille,
    // Neue Sätze (1.3)
    Punctuation, // rein aus Satzzeichen
    Symbols,     // rein aus Sonderzeichen
    // High Contrast (1.4)
    JNVSH, // Maximale Luminanz-Dynamik für audio-reaktive Effekte
}

impl Charset {
    pub const ALL: &'static [Charset] = &[
        Charset::Latin,
        Charset::Cyrillic,
        Charset::Hiragana,
        Charset::Katakana,
        Charset::Arabic,
        Charset::Braille,
        Charset::Punctuation,
        Charset::Symbols,
        Charset::JNVSH,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Charset::Latin => "Latin",
            Charset::Cyrillic => "Кирилл",
            Charset::Hiragana => "ひらがな",
            Charset::Katakana => "カタカナ",
            Charset::Arabic => "عربي",
            Charset::Braille => "Braille",
            Charset::Punctuation => "Satzzeichen",
            Charset::Symbols => "Sonderzeichen",
            Charset::JNVSH => "JNVSH",
        }
    }

    /// Zeichenbreite in Terminal-Spalten (wide chars = 2, narrow = 1)
    pub fn col_width(self) -> u32 {
        match self {
            Charset::Hiragana | Charset::Katakana => 2,
            _ => 1,
        }
    }

    /// Zeichen sortiert von leer (Index 0) nach dicht (letzter Index)
    pub fn chars(self) -> &'static [char] {
        match self {
            Charset::Latin => &[
                ' ', '.', '\'', '`', '^', ',', ':', ';', '-', '_', '~', '!', 'i', 'l', 'I', '|',
                '/', '\\', '(', ')', '[', ']', '{', '}', 'r', 't', 'f', 'j', '1', 'v', 'c', 'z',
                'x', 'n', 'u', 'o', 'e', 'a', 's', 'y', 'k', 'h', 'd', 'b', 'p', 'q', 'g', 'w',
                'm', '+', '=', '*', '#', '0', 'O', 'C', 'U', 'X', 'Z', 'L', 'J', 'Y', 'V', 'T',
                'F', 'E', 'P', 'S', 'A', 'G', 'K', 'H', 'D', 'B', 'R', 'N', 'Q', 'M', 'W', '%',
                '&', '8', '@', '$',
            ],
            Charset::Cyrillic => &[
                ' ', '·', 'і', 'ї', 'є', 'а', 'е', 'о', 'с', 'х', 'р', 'н', 'к', 'з', 'и', 'т',
                'г', 'д', 'у', 'ф', 'б', 'в', 'й', 'л', 'м', 'п', 'ц', 'ч', 'ш', 'э', 'ю', 'я',
                'Д', 'Ж', 'З', 'И', 'Й', 'Л', 'П', 'Ф', 'Ц', 'Ч', 'Ш', 'Щ', 'Э', 'Ю', 'Я', 'Б',
                'В', 'Г', 'Е', 'М', 'Н', 'Т', 'Х', 'Ъ', 'Ы', 'Ь', 'А', 'О', 'С', 'К', 'Р', 'У',
            ],
            Charset::Hiragana => &[
                ' ', 'あ', 'い', 'う', 'え', 'お', 'か', 'き', 'く', 'け', 'こ', 'さ', 'し', 'す',
                'せ', 'そ', 'た', 'ち', 'つ', 'て', 'と', 'な', 'に', 'ぬ', 'ね', 'の', 'は', 'ひ',
                'ふ', 'へ', 'ほ', 'ま', 'み', 'む', 'め', 'も', 'や', 'ゆ', 'よ', 'ら', 'り', 'る',
                'れ', 'ろ', 'わ', 'を', 'ん', 'が', 'ぎ', 'ぐ', 'げ', 'ご', 'ざ', 'じ', 'ず', 'ぜ',
                'ぞ', 'だ', 'ぢ', 'づ', 'で', 'ど',
            ],
            Charset::Katakana => &[
                ' ', 'ア', 'イ', 'ウ', 'エ', 'オ', 'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス',
                'セ', 'ソ', 'タ', 'チ', 'ツ', 'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ', 'ハ', 'ヒ',
                'フ', 'ヘ', 'ホ', 'マ', 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ', 'ラ', 'リ', 'ル',
                'レ', 'ロ', 'ワ', 'ヲ', 'ン', 'ガ', 'ギ', 'グ', 'ゲ', 'ゴ', 'ザ', 'ジ', 'ズ', 'ゼ',
                'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド',
            ],
            Charset::Arabic => &[
                ' ', '·', 'ء', 'آ', 'أ', 'إ', 'ا', 'ب', 'ت', 'ث', 'ج', 'ح', 'خ', 'د', 'ذ', 'ر',
                'ز', 'س', 'ش', 'ص', 'ض', 'ط', 'ظ', 'ع', 'غ', 'ف', 'ق', 'ك', 'ل', 'م', 'ن', 'ه',
                'و', 'ي', 'ى', 'ة', 'ئ', 'ؤ',
            ],
            Charset::Braille => &[
                ' ', '⠁', '⠂', '⠃', '⠄', '⠅', '⠆', '⠇', '⠈', '⠉', '⠊', '⠋', '⠌', '⠍', '⠎', '⠏',
                '⠐', '⠑', '⠒', '⠓', '⠔', '⠕', '⠖', '⠗', '⠘', '⠙', '⠚', '⠛', '⠜', '⠝', '⠞', '⠟',
                '⠠', '⠡', '⠢', '⠣', '⠤', '⠥', '⠦', '⠧', '⠨', '⠩', '⠪', '⠫', '⠬', '⠭', '⠮', '⠯',
                '⠰', '⠱', '⠲', '⠳', '⠴', '⠵', '⠶', '⠷', '⠸', '⠹', '⠺', '⠻', '⠼', '⠽', '⠾', '⠿',
            ],
            // 1.3: Rein aus Satzzeichen — sortiert nach visueller Dichte
            Charset::Punctuation => &[
                ' ', '.', ',', '\'', '`', '"', ':', ';', '!', '?', '-', '_', '~', '(', ')', '[',
                ']', '{', '}', '<', '>', '/', '\\', '|', '+', '=', '*', '^', '#', '&', '%', '@',
            ],
            // 1.3: Rein aus Sonderzeichen / Box-Drawing / Symbolen
            Charset::Symbols => &[
                ' ', '·', '°', '•', '○', '◦', '□', '△', '▷', '◁', '▽', '◇', '◈', '◉', '◌', '◍',
                '◎', '●', '◐', '◑', '◒', '◓', '◔', '◕', '◖', '◗', '★', '☆', '♦', '♠', '♣', '♥',
                '⊕', '⊗', '⊘', '⊙', '⊚', '⊛', '⊞', '⊟', '⊠', '⊡', '▪', '▫', '▬', '▭', '▮', '▯',
                '▰', '▱', '▲', '▴', '▶', '▸', '▼', '▾', '◀', '◂', '◆', '◈', '▉', '▊', '▋', '▌',
                '▍', '▎', '▏', '█',
            ],
            // 1.4: JNVSH - Maximale Luminanz-Dynamik
            // Kombiniert: leere Zeichen, Kanten-Zeichen, Blöcke
            // Optimiert für audio-reaktive Effekte mit maximalem Kontrast
            Charset::JNVSH => &[
                // Leichteste (Index 0-15): Minimal占用
                ' ', '.', '·', '`', '\'', ',', '´', '¨', ':', ';', '·', '•', '°', 'µ', '†', '‡',
                // Leicht (16-31): Dünne Linien
                '-', '_', '~', '¯', '´', 'ˋ', 'ˊ', 'ː', '∵', '∴', '⊙', '○', '◌', '◠', '◡', '⋆',
                // Mittel-Leicht (32-47): Feine Details
                '|', '¦', '╽', '╿', '╎', '╏', '┊', '┋', '/', '\\', '⁄', '∕', '╱', '╲', '⟋', '⟍',
                // Mittel (48-63): Balance
                '!', 'i', 'l', 'ı', 'ł', '|', '¦', 'ᵢ', '⌐', '¬', '½', '¼', '¡', '¿', '‽', '⁂',
                // Mittel-Dunkel (64-79): Erkennbare Formen
                '1', 'r', 't', 'f', 'j', 'v', 'c', 'z', 'n', 'u', 'o', 'e', 'a', 's', 'y', 'k',
                // Dunkel (80-95): Schwere Konturen
                'h', 'd', 'b', 'p', 'q', 'g', 'w', 'm', 'x', '*', '+', '‡', '™', '®', '©', '℗',
                // Dunkelste (96-111): Maximale Füllung
                '#', '█', '▓', '▒', '░', '▀', '▄', '▌', '▐', '▬', '▮', '▯', '▰', '▱', '▲', '●',
                // Extrem dunkel (112+): Blöcke
                '■', '□', '▪', '▫', '◆', '◇', '◉', '◈', '★', '☆', '♦', '♠', '♣', '♥', '♢', '♤',
            ],
        }
    }

    /// Gibt den effektiven Slice zurück, eingeengt durch contrast (0.0–1.0).
    /// contrast=1.0 → voller Satz; contrast=0.5 → mittlere 50% der Dichte-Range.
    /// Dadurch werden bei hohem Kontrast nur extreme (helle + dunkle) Zeichen genutzt,
    /// bei niedrigem Kontrast nur die mittlere Grauzone.
    pub fn slice_by_contrast<'a>(chars: &'a [char], contrast: f32) -> &'a [char] {
        let len = chars.len();
        if len < 3 {
            return chars;
        }
        let c = contrast.clamp(0.1, 1.0);
        // Bei contrast=1.0: voller Bereich [0..len]
        // Bei contrast=0.5: Mitte ±25% → [len/4 .. 3*len/4]
        let margin = ((1.0 - c) * 0.5 * len as f32) as usize;
        let lo = margin;
        let hi = (len - margin).max(lo + 2);
        &chars[lo..hi]
    }

    pub fn next(self) -> Charset {
        let idx = Self::ALL.iter().position(|&c| c == self).unwrap_or(0);
        Self::ALL[(idx + 1) % Self::ALL.len()]
    }

    /// Characters classified by detail level:
    /// - Fine: thin strokes, dots, lines (best for edges)
    /// - Medium: balanced density
    /// - Coarse: heavy fills, blocks (best for solid areas)
    pub fn chars_by_detail(self) -> (Vec<char>, Vec<char>, Vec<char>) {
        match self {
            Charset::Latin => {
                let fine = vec![
                    '.', ',', '\'', '`', '^', ':', ';', '-', '_', '~', '!', 'i', 'l', '|', 'r',
                    't', 'f', 'j', '1', 'v', 'c', 'z',
                ];
                let medium = vec![
                    '/', '\\', '(', ')', '[', ']', '{', '}', 'x', 'n', 'u', 'o', 'e', 'a', 's',
                    'y', 'k', 'h', 'd', 'b', 'p', 'q', 'g', 'w', 'm',
                ];
                let coarse = vec![
                    '+', '=', '*', '#', '0', 'O', 'C', 'U', 'X', 'Z', 'L', 'J', 'Y', 'V', 'T', 'F',
                    'E', 'P', 'S', 'A', 'G', 'K', 'H', 'D', 'B', 'R', 'N', 'Q', 'M', 'W', '%', '&',
                    '8', '@', '$',
                ];
                (fine, medium, coarse)
            }
            Charset::Cyrillic => {
                let fine = vec![' ', '·', 'і', 'ї', 'є', 'а', 'е', 'о', 'с'];
                let medium = vec![
                    'х', 'р', 'н', 'к', 'з', 'и', 'т', 'г', 'д', 'у', 'ф', 'б', 'в', 'й', 'л', 'м',
                    'п', 'ц', 'ч', 'ш',
                ];
                let coarse = vec![
                    'э', 'ю', 'я', 'Д', 'Ж', 'З', 'И', 'Й', 'Л', 'П', 'Ф', 'Ц', 'Ч', 'Ш', 'Щ', 'Э',
                    'Ю', 'Я', 'Б', 'В', 'Г', 'Е', 'М', 'Н', 'Т', 'Х', 'Ъ', 'Ы', 'Ь', 'А', 'О', 'С',
                    'К', 'Р', 'У',
                ];
                (fine, medium, coarse)
            }
            Charset::Hiragana => {
                let fine = vec![' ', 'あ', 'い', 'う', 'え', 'お'];
                let medium = vec![
                    'か', 'き', 'く', 'け', 'こ', 'さ', 'し', 'す', 'せ', 'そ', 'た', 'ち', 'つ',
                    'て', 'と', 'な', 'に', 'ぬ', 'ね', 'の',
                ];
                let coarse = vec![
                    'は', 'ひ', 'ふ', 'へ', 'ほ', 'ま', 'み', 'む', 'め', 'も', 'や', 'ゆ', 'よ',
                    'ら', 'り', 'る', 'れ', 'ろ', 'わ', 'を', 'ん', 'が', 'ぎ', 'ぐ', 'げ', 'ご',
                    'ざ', 'じ', 'ず', 'ぜ', 'ぞ', 'だ', 'ぢ', 'づ', 'で', 'ど',
                ];
                (fine, medium, coarse)
            }
            Charset::Katakana => {
                let fine = vec![' ', 'ア', 'イ', 'ウ', 'エ', 'オ'];
                let medium = vec![
                    'カ', 'キ', 'ク', 'ケ', 'コ', 'サ', 'シ', 'ス', 'セ', 'ソ', 'タ', 'チ', 'ツ',
                    'テ', 'ト', 'ナ', 'ニ', 'ヌ', 'ネ', 'ノ',
                ];
                let coarse = vec![
                    'ハ', 'ヒ', 'フ', 'ヘ', 'ホ', 'マ', 'ミ', 'ム', 'メ', 'モ', 'ヤ', 'ユ', 'ヨ',
                    'ラ', 'リ', 'ル', 'レ', 'ロ', 'ワ', 'ヲ', 'ン', 'ガ', 'ギ', 'グ', 'ゲ', 'ゴ',
                    'ザ', 'ジ', 'ズ', 'ゼ', 'ゾ', 'ダ', 'ヂ', 'ヅ', 'デ', 'ド',
                ];
                (fine, medium, coarse)
            }
            Charset::Arabic => {
                let fine = vec![' ', '·', 'ء', 'آ', 'أ', 'إ', 'ا'];
                let medium = vec![
                    'ب', 'ت', 'ث', 'ج', 'ح', 'خ', 'د', 'ذ', 'ر', 'ز', 'س', 'ش', 'ص', 'ض', 'ط', 'ظ',
                    'ع', 'غ',
                ];
                let coarse = vec![
                    'ف', 'ق', 'ك', 'ل', 'م', 'ن', 'ه', 'و', 'ي', 'ى', 'ة', 'ئ', 'ؤ',
                ];
                (fine, medium, coarse)
            }
            Charset::Braille => {
                let fine = vec![
                    ' ', '⠁', '⠂', '⠃', '⠄', '⠅', '⠆', '⠇', '⠈', '⠉', '⠊', '⠋', '⠌', '⠍', '⠎', '⠏',
                ];
                let medium = vec![
                    '⠐', '⠑', '⠒', '⠓', '⠔', '⠕', '⠖', '⠗', '⠘', '⠙', '⠚', '⠛', '⠜', '⠝', '⠞', '⠟',
                ];
                let coarse = vec![
                    '⠠', '⠡', '⠢', '⠣', '⠤', '⠥', '⠦', '⠧', '⠨', '⠩', '⠪', '⠫', '⠬', '⠭', '⠮', '⠯',
                    '⠰', '⠱', '⠲', '⠳', '⠴', '⠵', '⠶', '⠷', '⠸', '⠹', '⠺', '⠻', '⠼', '⠽', '⠾', '⠿',
                ];
                (fine, medium, coarse)
            }
            Charset::Punctuation => {
                let fine = vec![' ', '.', ',', '\'', '`', '"', ':', ';', '-', '_', '~'];
                let medium = vec![
                    '!', '?', '(', ')', '[', ']', '{', '}', '<', '>', '/', '\\', '|',
                ];
                let coarse = vec!['+', '=', '*', '^', '#', '&', '%', '@'];
                (fine, medium, coarse)
            }
            Charset::Symbols => {
                let fine = vec![' ', '·', '°', '•', '○', '◦', '□', '△', '▷'];
                let medium = vec![
                    '◁', '▽', '◇', '◈', '◉', '◌', '◍', '◎', '●', '◐', '◑', '◒', '◓', '◔', '◕', '◖',
                    '◗',
                ];
                let coarse = vec![
                    '★', '☆', '♦', '♠', '♣', '♥', '⊕', '⊗', '⊘', '⊙', '⊚', '⊛', '⊞', '⊟', '⊠', '⊡',
                    '▪', '▫', '▬', '▭', '▮', '▯', '▰', '▱', '▲', '▴', '▶', '▸', '▼', '▾', '◀', '◂',
                    '◆', '█',
                ];
                (fine, medium, coarse)
            }
            Charset::JNVSH => {
                // JNVSH: Speziell für maximale Kontrast-Dynamik
                let fine = vec![
                    ' ', '.', '·', '`', '\'', ',', '´', '¨', ':', ';', '•', '°', 'µ', '†', '‡',
                    '-', '_', '~', '¯', 'ː', '∵', '∴', '⊙', '○',
                ];
                let medium = vec![
                    '|', '¦', '╽', '╿', '╎', '╏', '┊', '┋', '/', '\\', '⁄', '∕', '╱', '╲', '⟋',
                    '⟍', '!', 'i', 'l', 'ı', 'ł', '⌐', '¬', '½', '¼', '¡', '¿', '‽', '⁂', '1', 'r',
                    't',
                ];
                let coarse = vec![
                    'h', 'd', 'b', 'p', 'q', 'g', 'w', 'm', 'x', '*', '+', '‡', '™', '®', '©', '℗',
                    '#', '█', '▓', '▒', '░', '▀', '▄', '▌', '▐', '▬', '▮', '▯', '▰', '▱', '▲', '●',
                    '■', '□', '▪', '▫', '◆', '◇', '◉', '◈', '★', '☆', '♦', '♠', '♣', '♥', '♢', '♤',
                ];
                (fine, medium, coarse)
            }
        }
    }

    pub fn detail_chars(self, detail: DetailLevel, contrast: f32) -> Vec<char> {
        let (fine, medium, coarse) = self.chars_by_detail();
        let c = contrast.clamp(0.1, 1.0);

        let chars = match detail {
            DetailLevel::Fine => fine,
            DetailLevel::Medium => medium,
            DetailLevel::Coarse => coarse,
        };

        let len = chars.len();
        let margin = ((1.0 - c) * 0.5 * len as f32) as usize;
        let lo = margin.min(len.saturating_sub(1));
        let hi = (len - margin).max(lo + 2).min(len);

        chars[lo..hi].to_vec()
    }
}
