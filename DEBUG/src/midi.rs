use midir::{MidiInput, MidiInputConnection};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct MidiState {
    pub note: u8,
    pub velocity: u8,
    pub cc: [f32; 128],
    pub pitch_bend: f32,
    pub modulation: f32,
}

impl Default for MidiState {
    fn default() -> Self {
        MidiState {
            note: 0,
            velocity: 0,
            cc: [0.0; 128],
            pitch_bend: 0.0,
            modulation: 0.0,
        }
    }
}

pub type SharedMidi = Arc<Mutex<MidiState>>;

pub struct MidiHandler {
    connection: Option<MidiInputConnection<()>>,
    shared: SharedMidi,
}

impl MidiHandler {
    pub fn new() -> Self {
        let shared: SharedMidi = Arc::new(Mutex::new(MidiState::default()));

        let midi_in = match MidiInput::new("ascii-cam") {
            Ok(m) => m,
            Err(e) => {
                eprintln!("MIDI init error: {}", e);
                return MidiHandler {
                    connection: None,
                    shared,
                };
            }
        };

        let ports = midi_in.ports();
        if ports.is_empty() {
            eprintln!("No MIDI devices found");
            return MidiHandler {
                connection: None,
                shared,
            };
        }

        let port = &ports[0];
        let port_name = midi_in.port_name(port).unwrap_or_else(|_| "Unknown".into());
        eprintln!("MIDI input: {}", port_name);

        let shared_clone = shared.clone();

        let connection = match midi_in.connect(
            port,
            "ascii-cam-input",
            move |_stamp, message, _| {
                if let Some(midi_msg) = parse_midi(message) {
                    let mut state = shared_clone.lock().unwrap();
                    match midi_msg {
                        MidiMessage::NoteOn(note, velocity) => {
                            state.note = note;
                            state.velocity = velocity;
                        }
                        MidiMessage::NoteOff(note) => {
                            if state.note == note {
                                state.velocity = 0;
                            }
                        }
                        MidiMessage::ControlChange(cc, value) => {
                            state.cc[cc as usize] = value as f32 / 127.0;
                        }
                        MidiMessage::PitchBend(value) => {
                            state.pitch_bend = (value as f32 - 8192.0) / 8192.0;
                        }
                        MidiMessage::Modulation(value) => {
                            state.modulation = value as f32 / 127.0;
                        }
                        MidiMessage::Unknown => {}
                    }
                }
            },
            (),
        ) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("MIDI connect error: {}", e);
                None
            }
        };

        MidiHandler { connection, shared }
    }

    pub fn get_state(&self) -> SharedMidi {
        self.shared.clone()
    }

    pub fn has_device(&self) -> bool {
        self.connection.is_some()
    }
}

impl Default for MidiHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MidiHandler {
    fn drop(&mut self) {
        self.connection = None;
    }
}

enum MidiMessage {
    NoteOn(u8, u8),
    NoteOff(u8),
    ControlChange(u8, u8),
    PitchBend(u16),
    Modulation(u8),
    Unknown,
}

fn parse_midi(message: &[u8]) -> Option<MidiMessage> {
    if message.is_empty() {
        return None;
    }

    let status = message[0] & 0xF0;
    let _channel = message[0] & 0x0F;

    match status {
        0x90 if message.len() >= 3 => {
            if message[2] > 0 {
                Some(MidiMessage::NoteOn(message[1], message[2]))
            } else {
                Some(MidiMessage::NoteOff(message[1]))
            }
        }
        0x80 if message.len() >= 3 => Some(MidiMessage::NoteOff(message[1])),
        0xB0 if message.len() >= 3 => {
            let cc = message[1];
            let value = message[2];
            if cc == 1 {
                Some(MidiMessage::Modulation(value))
            } else {
                Some(MidiMessage::ControlChange(cc, value))
            }
        }
        0xE0 if message.len() >= 3 => {
            let value = ((message[2] as u16) << 7) | (message[1] as u16);
            Some(MidiMessage::PitchBend(value))
        }
        _ => Some(MidiMessage::Unknown),
    }
}

#[allow(dead_code)]
pub fn list_midi_devices() -> Vec<String> {
    MidiInput::new("ascii-cam")
        .map(|midi| {
            midi.ports()
                .into_iter()
                .filter_map(|p| midi.port_name(&p).ok())
                .collect()
        })
        .unwrap_or_default()
}
