# ascii-cam

## Projektbeschreibung
Ein performanter Echtzeit-ASCII-Renderer, der Kamera-Inputs in prozessierte Text-Grafiken umwandelt. Das Projekt nutzt fortschrittliche Shader-Techniken zur Kantenerkennung und eine audio-reaktive Steuerung zur Manipulation von Bildparametern.

---

##  To-Do / Roadmap

### 1. Bildverarbeitung & Shader-Logik
* **Kanten-Erkennung verfeinern:** Implementierung eines Hybrid-Shaders nach dem Acerola-Prinzip:
    * **Tiling:** Unterteilung des Inputs in $8 \times 8$ Pixel Kacheln.
    * **Luminanz-Mapping:** Auswahl der Basis-Zeichen basierend auf der Helligkeit (z. B. `.` für dunkle, `W` für helle Bereiche).
    * **Sobel-Filter:** Nutzung eines Sobel-Operators zur Extraktion von Kanten und deren Winkeln.
    * **Richtungs-Mapping:** Zuweisung spezifischer ASCII-Zeichen basierend auf dem Kantenwinkel:
        * Horizontal: `_`
        * Vertikal: `|`
        * Diagonal: `/` oder `\`.
    * **Stabilisierung:** Einsatz eines **Compute-Shaders**, der ein Histogramm pro Kachel erstellt, um das dominanteste Merkmal (Kante vs. Fläche) zu bestimmen und Flimmern zu reduzieren.
* **Zeichensatz-Optimierung:** Reduzierung der verfügbaren Glyphen, um den Gesamtkontrast des Bildes zu erhöhen und visuelles Rauschen zu minimieren.

### 2. Audio-Engine & BPM-Sync
* **Main Gain Regulator:** Implementierung eines globalen Audio-Eingangsreglers mit Unterstützung für Verstärkung und Absenkung (Dämpfung).
* **BPM-Stabilisierung:** Umstellung auf ein "Frame-Burst"-Verfahren. Statt Einzel-Frames bei harten BPM-Peaks zu rendern, wird eine Sequenz von Frames pro Beat generiert, um visuelle Ruckler bei Schwankungen abzufangen.
* **Band-Mapping:** Jedes der drei Frequenzbänder (`1`, `2`, `3`) muss eine Ziel-Auswahl (Target) erhalten:
    * Layer-Kontrast
    * Farbintensität (Sättigung der Layer)
    * Kamera-Hardware-Schnittstellen (z. B. Fokus-Shift).

### 3. Hardware-Steuerung (Camera API)
* **Rauschunterdrückung:** Integration nativer Kamera-Hardware-Filter zur Vorreinigung des Signals.
* **Parameter-Mapping:** Prüfung und Implementierung von Schnittstellen zur Steuerung von Fokus, Belichtung und ISO direkt über die Audio-Bänder.

### 4. User Interface & Steuerung
* **Neues Steuersystem:** Das alte System wird verworfen. Implementierung dedizierter Tastenpaare:
	- Anstatt mit `+` und `-` ausgewählte parameter zu bearbeiten will ich für jede option 2 dedizierte keys haben.
	- ordne sie sinnvoll und nah bei einander an.
    * `1` / `2` / `3`: Auswahl des aktiven Frequenzbandes.
    * `x` : als exit der bänder wird verworfen, wir erlauben einen direkten wechsel.
    * `g` : Bringt uns in den Globalen modus zurück.
    * `Q`: Programm sicher beenden (Quit).
* **Cross-Platform Support:** Code-Refactoring für Kompatibilität mit **macOS** (Metal/CoreVideo) und **Windows** (DirectX/MediaFoundation) zusätzlich zu Linux.

---

## Dokumentation (Geplant)
Nach Abschluss der Entwicklung wird eine detaillierte `README.md` erstellt, die folgende Punkte umfasst:
* **Technische Spezifikationen:** Details zur Shader-Pipeline und Audio-Analyse.
* **Dependencies:** Liste aller Bibliotheken (z. B. OpenCV, PortAudio, GLFW).
* **Build-Anleitung:** Schritt-für-Schritt-Guide für das Compilieren unter Linux, Windows (MSVC/MinGW) und macOS (Xcode/Clang).
* **Keymap-Table:** Übersicht aller dedizierten Tastenbelegungen.

---

### Tipps für die Umsetzung:
* **Für den Fokus:** Unter macOS nutzt du am besten `AVCaptureDevice`, unter Windows `MediaFoundation`, um auf den Fokus zuzugreifen. Nicht jede Webcam unterstützt manuellen Fokus via Software.
* **Für die Performance:** Da du Compute-Shader für das Histogramm nutzt, achte darauf, dass die GPU-Buffer effizient zwischen dem Grafik- und dem Compute-Pass geteilt werden.
