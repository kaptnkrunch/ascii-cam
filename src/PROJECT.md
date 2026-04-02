# ascii-cam

**ascii-cam** ist ein hochperformanter Echtzeit-ASCII-Video-Renderer, der Videostreams in dynamische Text-Art umwandelt. Das Projekt kombiniert klassische Grafik-Algorithmen mit moderner GPU-Beschleunigung und einer audio-reaktiven Engine, um eine Brücke zwischen der Ästhetik der 2000er Game-FAQs und moderner VJ-Software zu schlagen.

## Kern-Features

### Fortschrittliche Shader-Pipeline
* **8x8 Tiling & Luminanz:** Das Eingangsbild wird in $8 \times 8$ Pixel große Kacheln unterteilt, deren Helligkeit (Luminanz) die Wahl des Basis-ASCII-Zeichens bestimmt.
* **Richtungsbasierte Kantenerkennung:** Implementierung eines Sobel-Filters, der nicht nur Kanten erkennt, sondern auch deren Winkel berechnet.
* **Intelligentes Character-Mapping:** Kanten werden präzise durch spezifische Zeichen dargestellt:
    * Horizontal: `_`
    * Vertikal: `|`
    * Diagonal: `/` oder `\`
* **GPU-Histogramm-Verfeinerung:** Ein Compute-Shader analysiert jede Kachel via Histogramm, um das visuell dominanteste Merkmal zu bestimmen und Linien zu stabilisieren.
* **Kontrast-Optimierung:** Verwendung eines reduzierten Zeichensatzes, um den maximalen visuellen Kontrast zu gewährleisten.

### 🔊 Audio-Reaktivität & BPM-Engine
* **Drei-Band-Analyse:** Dedizierte Steuerung über drei Frequenzbänder (Low, Mid, High), wählbar über die Tasten `1`, `2` und `3`.
* **Modulares Target-Mapping:** Jedes Band kann wahlweise den Layer-Kontrast, die Farbintensität oder Hardware-Parameter steuern.
* **Main Gain Regler:** Ein globaler Gain-Regler ermöglicht die präzise Verstärkung oder Absenkung des Eingangssignals.
* **BPM Frame Burst:** Ein spezialisierter Rendering-Modus, der bei BPM-Peaks mehrere Frames in einem Burst generiert, um Schwankungen in der Erkennung auszugleichen und ein flüssiges Bild zu garantieren.

### Hardware-Integration
* **Kamera-Interface:** Native Unterstützung für Fokus-Steuerung und Rauschunterdrückung auf Hardware-Ebene.
* **Cross-Platform:** Volle Kompatibilität mit Linux, Windows und macOS.

---

## Steuerung (Keymap)

### Globaler Modus

| Taste | Aktion |
| :--- | :--- |
| `Q` | Programm beenden (Quit) — aus jedem Modus |
| `1` / `2` / `3` | Band-Modus wechseln — direkt aus jedem Modus |
| `g` | Zurück zu Global — aus jedem Modus |
| `0` | Base-Layer-Modus |
| `Space` | Audio-Device-Menü |
| `r` / `t` | Globaler Kontrast erhöhen / senken |
| `f` / `h` | Energie-Reaktion erhöhen / senken |
| `b` / `v` | Char-Size vergrößern / verkleinern |
| `c` / `z` | Palette vorwärts / rückwärts |
| `i` | Invertieren (toggle) |
| `y` | BPM-Sync (toggle) |

### Band-Modus (`1` / `2` / `3`)

| Taste | Aktion |
| :--- | :--- |
| `r` / `t` | Energy-Scale erhöhen / senken |
| `f` / `h` | Contrast-Lo erhöhen / senken |
| `b` / `n` | Contrast-Hi erhöhen / senken |
| `l` | Layer durchschalten |
| `s` | Schriftsystem durchschalten |
| `m` | Modus durchschalten (`+add` / `-sub` / `~inv`) |
| `o` / `p` | Farb-Override rückwärts / vorwärts |
| `u` | Mute (toggle) |

### Base-Layer-Modus (`0`)

| Taste | Aktion |
| :--- | :--- |
| `s` | Schriftsystem durchschalten |
| `v` | Base-Layer an/aus |
| `r` / `t` | Kontrast erhöhen / senken |

---

## Technische Details

### Shader-Logik (Sobel & Winkel)
Der Gradient wird über den Sobel-Operator berechnet:
$$G = \sqrt{G_x^2 + G_y^2}$$
Der Winkel $\theta$ für die Zeichenzuweisung ergibt sich aus:
$$\theta = \operatorname{atan2}(G_y, G_x)$$

### Backend-Architektur
* **Windows:** Media Foundation für Low-Level Kamera-Zugriff.
* **macOS:** AVFoundation / AVCaptureDevice Schnittstelle.
* **Linux:** V4L2 (Video4Linux).

---

## Installation & Build

### Voraussetzungen
* **Rust** (stable, >= 1.75)
* **ALSA / PipeWire** (Linux Audio)
* **V4L2** (Kamera)

### Build-Anleitung

```bash
git clone https://github.com/kaptnkrunch/ascii-cam
cd ascii-cam
cargo build --release
```

Zeichengröße im Terminal vorher verkleinern mit `SHIFT+STRG+-`, dann:

```bash
./target/release/ascii-cam
```

### System-Audio (Loopback)
Für Musik vom System: in PipeWire ein `.monitor`-Device wählen.  
Geht am einfachsten mit `pavucontrol` oder `pw-link`, oder direkt über das Audio-Menü im Programm (`Space`).
