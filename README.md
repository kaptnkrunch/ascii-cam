**Technischer Plan:**

<img src="https://github.com/kaptnkrunch/ascii-cam/blob/main/ascii_cam_audio_architecture.svg">

**Neue Crates:**
- `cpal` — plattformübergreifendes Audio-I/O (ALSA/PipeWire auf Arch)
- `rustfft` — FFT für Frequenzanalyse
- `arc-mutex` über Rust std — geteilter State zwischen Audio-Thread und Render-Thread

**Parallelität:**
- Audio-Thread läuft mit CPAL kontinuierlich, füllt einen Ringbuffer
- Render-Thread liest `Arc<Mutex<AudioState>>` mit den 3 Band-Energiewerten
- Kein Blocking, da der Render-Tick sowieso ~30fps ist
Aktuelles Mapping wird in der Statuszeile angezeigt

**Schriftsysteme** (Unicode-Blöcke als Zeichensätze):
- Latin (Standard ASCII)
- Kyrillisch (`А Б В ... Я`)
- Hiragana / Katakana
- Arabisch (RTL, aber als Zeichen-Dichte verwendbar)
- Braille (sehr dicht)

**Steuerung zur Laufzeit:**

| Taste | Aktion |
|-------|--------|
| `1` / `2` / `3` | Band auswählen (Bass / Mid / High) |
| `m` | Mapping des aktiven Bands durchschalten: `—` → `Kontrast` → `Auflösung` → `Schrift` → `Dichte` |
| `s` | Script manuell wechseln (Latin → Кирилл → ひらがな → カタカナ → عربي → Braille) |
| `i` | Invertieren |
| `+` / `-` | Auflösung manuell |
| `q` | Beenden |

---
**Bänder:**

Die Bänder sind vorerst fix

|-------|--------|
| BÄSSE | 20Hz - 90Hz |
| MITTEN | 1400Hz - 4000Hz |
| HÖHEN | 7000Hz - 20000Hz |

**Wie das Script-Mapping funktioniert:** Es triggert nicht kontinuierlich, sondern nur auf der **steigenden Flanke** über 0.75 Energie — also wechselt das Schriftsystem einmal pro Transient (z.B. jeder Schlagzeugschlag), nicht dauerhaft flackernd.

**System-Audio (Loopback):** Wenn du Musik vom System capturen willst, wähle in PipeWire ein `.monitor`-Device. Das geht am einfachsten mit `pavucontrol` oder `pw-link`.
