
## 📄 skill_shader_pipeline.md
**Fokus:** Mathematische Umsetzung der ASCII-Generierung und Kantenstabilität.

### 1. Luminanz & Tiling
Jeder Frame wird in $8 \times 8$ Pixel Blöcke unterteilt. Die durchschnittliche Luminanz $L$ berechnet sich nach der Formel:
$$L = 0.2126R + 0.7152G + 0.0722B$$
Diese $L$ wird auf einen reduzierten Zeichensatz (z.B. 10 Stufen) gemappt, um den Kontrast zu wahren.

### 2. Sobel-Kanten-Analyse
Zur Bestimmung der Kantenrichtung nutzen wir zwei Kerne für die Gradienten $G_x$ und $G_y$:
$$G_x = \begin{bmatrix} -1 & 0 & 1 \\ -2 & 0 & 2 \\ -1 & 0 & 1 \end{bmatrix}, \quad G_y = \begin{bmatrix} -1 & -2 & -1 \\ 0 & 0 & 0 \\ 1 & 2 & 1 \end{bmatrix}$$
Der Winkel $\theta$ wird über $\operatorname{atan2}(G_y, G_x)$ berechnet und in vier Quadranten für die Zeichen `_`, `|`, `/`, `\` unterteilt.



### 3. Compute Shader Histogramm
Um Flimmern zu vermeiden, schreibt jeder Thread einer $8 \times 8$ Gruppe sein Ergebnis in den **Group Shared Memory**. Der erste Thread der Gruppe wertet das Histogramm aus:
* Wenn die Anzahl der Kantenpixel einen Schwellenwert $T$ überschreitet, wird das dominierende Richtungszeichen gewählt.
* Andernfalls wird das Luminanz-Zeichen verwendet.

---

## 📄 skill_audio_hardware.md
**Fokus:** Audio-Reaktivität, BPM-Stabilisierung und Kamera-API.

### 1. Frame Burst BPM Sync
Um das Haken des Bildes bei BPM-Peaks zu lösen, implementieren wir einen **Ringbuffer für Frames**:
* **Peak erkannt:** Das System rendert nicht nur einen Frame, sondern triggert eine Sequenz von $N$ Frames mit interpolierten Parametern (Burst).
* Dies glättet visuelle Mikroruckler, wenn die BPM-Erkennung zwischen zwei Werten schwankt.

### 2. Audio Mapping & Gain
* **Main Gain:** Ein Pre-Processing Step multipliziert das Amplitudenspektrum mit einem Faktor $k$ ($0.0 < k < 2.0$), um auch eine Absenkung zu ermöglichen.
* **Target Mapping:** Jedes Band (Low, Mid, High) erhält einen Pointer auf ein Ziel-Attribut:
    * `Band[0] -> Camera.Focus`
    * `Band[1] -> Shader.Saturation`
    * `Band[2] -> Shader.Contrast`

### 3. Hardware Schnittstellen (Cross-Platform)
* **Windows:** Zugriff über `MediaFoundation` für Fokus- und Belichtungssteuerung.
* **macOS:** Nutzung von `AVCaptureDevice` (Objective-C/Swift Bridge), um hardwareseitige Rauschunterdrückung zu aktivieren.
* **Linux:** `v4l2-ctl` Integration für direkten Hardware-Zugriff.

---

## 📄 skill_interface_ux.md
**Fokus:** Steuerungsschema und plattformübergreifender Build.

### 1. Globales Key-Mapping
Das System nutzt einen zustandsbasierten Input-Handler:
* **Band-Selektion:** Tasten `1`, `2`, `3`.
* **Parameter-Modifikation:**
    * `+` (Numpad oder Shift-Gleich): Wert erhöhen.
    * `-` (Bindestrich): Wert verringern.
* **System:** `Q` für Clean-Exit (Release aller Hardware-Handles).

### 2. Zeichensatz-Reduktion
Um den Kontrast zu maximieren, wird der Standard-ASCII-Satz auf eine High-Contrast-Variante reduziert:
`[Leerzeichen, '.', ':', '-', '=', '+', '*', '#', '%', '@']`

### 3. Cross-Platform Build System (CMake)
Die `CMakeLists.txt` muss folgende plattformspezifische Backends laden:
* **Linux:** `pkg_check_modules(ALSA REQUIRED alsa)`
* **Windows:** `find_package(DirectX REQUIRED)`
* **macOS:** `find_library(COREVIDEO CoreVideo REQUIRED)`

---
