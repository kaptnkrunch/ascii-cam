# To-Do Liste

## 1. Bessere Hardware-Zugriffssteuerung
* Infrarot-Kanal auslesen
* Kamera-Fokus-Steuerung
* Belichtungszeit anpassen

## 2. Rendering-Algorithmus verfeinern
* Den Infrarot-Input als zusätzliche Ebene (Layer) nutzbar machen.
* Zusätzliche Modi hinzufügen: `Multiplikation`, `Division`, `XOR`, `XAND`.
* Die Rendering-Pipeline so umstellen, dass das Layering vor der „Skriptifizierung“ geschieht.
* Ein Layer-Option-Menü implementieren, um die verschiedenen Ebenen vor dem Rendering anzuordnen.
* Nutzung der Hardware-Daten zum Filtern feiner Details, die je nach Skript mit einem Detail-Schriftsatz dargestellt werden.

## 3. Verbesserung des BPM-Counters und des Sync-Modus
* Stabilität des Sync-Modus optimieren (aktuell noch sehr instabil).
* Implementierung einer Gewichtung, die aufeinanderfolgende, konstante Schläge stärker wertet als kurze Abweichungen (z. B. um Tempoverlust bei Breaks zu vermeiden).

## 4. Implementierung von MIDI- und OSC-Schnittstellen
* MIDI-Schnittstelle und Routing-Framework.
* OSC-Protokoll-Implementierung.
* MIDI über OSC.

## 5. X-Plattform-Bereitstellung
### 5.1 Windows
* Auflösung von Abhängigkeiten
* Build-Anleitung erstellen
* Readme.md aktualisieren

## 6. UI/UX Verbesserungen

### Quit Confirmation (USERFINDINGS Priority)
- [ ] Add `q` confirmation prompt
- [ ] Add countdown timer (5 seconds)
- [ ] Keep `Q` as instant quit

### Separate Control Window (USERFINDINGS Priority)
- [ ] Create separate control window
- [ ] Move UI controls to new window
- [ ] Keep terminal for ASCII only
- [ ] Add full-screen mode for values
- [ ] Implement window communication

### MIDI/OSC Fixes (USERFINDINGS Priority)
- [ ] Fix MIDI menu crash - add try-catch and null checks
- [ ] Add graceful fallback when no MIDI device available
- [ ] Add `g` key to MIDI menu
- [ ] Add `g` key to OSC menu
- [ ] Add `g` key to Camera/Audio menus

### BPM Stabilität (USERFINDINGS Priority)
- [ ] Increase beat history buffer (16 → 32)
- [ ] Add confidence threshold filtering (reject < 60%)
- [ ] Implement adaptive tempo smoothing
- [ ] Add double-beat detection
- [ ] Weight recent beats more heavily
