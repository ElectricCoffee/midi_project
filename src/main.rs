#![allow(dead_code, unused_macros)]

// imports
use std::fmt;

// constants
/// The duration of a whole note in MIDI ticks.
const MIDI_TEMPO: f32 = 3840.0;

// traits

// this trait is a workaround to make cloning Box<MusicElement> work
// this is required due to a quirk in the compiler
trait Cloneable {
    fn clone_box(&self) -> Box<MusicElement>;
}

impl<T> Cloneable for T where T: 'static + MusicElement + Clone {
    fn clone_box(&self) -> Box<MusicElement> {
        Box::new(self.clone())
    }
}

/// All `MusicElement`s need the ability to output a duration and set the channel retroactively
trait MusicElement : fmt::Debug + Cloneable {
    /// Returns the musical duration of a `MusicElement`
    /// Durations are implemented as `f32` due to the fractional nature of music.
    fn duration(&self) -> f32;
    /// Sets the channel to something else.
    fn set_channel(&mut self, channel: Channel);
}

impl Clone for Box<MusicElement> {
    fn clone(&self) -> Box<MusicElement> {
        self.clone_box()
    }
}

// enums
#[derive(Debug, Clone)]
/// Specifies the different note names available
/// TODO: implement From<String> on this, to allow things like "Do".into() or "C#".into()
enum NoteClass {
    C, Cs, D, Ds, E, F, Fs, G, Gs, A, As, B
}

impl fmt::Display for NoteClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            NoteClass::C  => write!(f, "C"),
            NoteClass::Cs => write!(f, "C#"),
            NoteClass::D  => write!(f, "D"),
            NoteClass::Ds => write!(f, "D#"),
            NoteClass::E  => write!(f, "E"),
            NoteClass::F  => write!(f, "F"),
            NoteClass::Fs => write!(f, "F#"),
            NoteClass::G  => write!(f, "G"),
            NoteClass::Gs => write!(f, "G#"),
            NoteClass::A  => write!(f, "A"),
            NoteClass::As => write!(f, "A#"),
            NoteClass::B  => write!(f, "B")
        }
    }
}

#[derive(Debug, Clone)]
/// Channel lists the available musical channels to play on
/// This list is the only instruments used in the original project, and thus do not contain the full list of instruments
enum Channel {
    Piano, Organ, Guitar, Violin, Flute, Trumpet, Helicopter, Telephone
}

impl Channel {
    /// Converts the channel to a number
    fn to_u32(self) -> u32 {
        self as u32 + 1
    }
}

// structs
#[derive(Debug)]
struct MidiNote {
    pitch: u32,
    duration: f32,
}

#[derive(Debug, Clone)]
/// The basic `Note` struct, which contains all the relevant information
struct Note {
    name: NoteClass,
    octave: u32,
    instrument: Channel,
    duration: f32,
}

impl Note {
    /// Creates a new note with the instrument set to `Piano` and duration to 1/4 by default.
    fn new(name: NoteClass, octave: u32) -> Note {
        Note { name, octave, instrument: Channel::Piano, duration: 0.25 }
    }

    /// Builder method that adds a channel to a `Note`
    fn channel(mut self, instrument: Channel) -> Note {
        self.instrument = instrument;
        self
    }

    /// Builder method that adds a duration to a `Note`
    fn duration(mut self, duration: f32) -> Note {
        self.duration = duration;
        self
    }

    /// Converts a `Note` to a `MidiNote`
    fn to_midi(&self) -> MidiNote {
        let offset = self.name.clone() as u32;
        let pitch = (12 * self.octave) + offset;
        MidiNote { pitch, duration: self.duration() }
    }
}

impl MusicElement for Note {
    fn duration(&self) -> f32 {
        self.duration * MIDI_TEMPO
    }

    fn set_channel(&mut self, channel: Channel) {
        self.instrument = channel;
    }
}

#[derive(Debug, Clone)]
/// A simple pause, only contains a duration
struct Pause {
    duration: f32,
}

impl Pause {
    /// Initialises a pause with a duration
    fn new(duration: f32) -> Pause {
        Pause { duration }
    }
}

impl MusicElement for Pause {
    fn duration(&self) -> f32 {
        self.duration * MIDI_TEMPO
    }

    fn set_channel(&mut self, _: Channel) {} // do nothing
}

#[derive(Debug, Default, Clone)]
/// Defines a sequential composition --- a series of notes next to each other
struct Sequential {
    elements: Vec<Box<MusicElement>>,
}

impl MusicElement for Sequential {
    fn duration(&self) -> f32 {
        self.elements.iter().map(|e| e.duration()).sum()
    }

    fn set_channel(&mut self, channel: Channel) {
        for element in self.elements.iter_mut() {
            element.set_channel(channel.clone());
        }
    }
}

/// Helper macro that boxes all its inputs before passing them into the vector
macro_rules! sequence {
    ($($e:expr),*) => ({
        let mut seq = Sequential::default();
        $(
            seq.elements.push(Box::new($e));
        )*
        seq
    })
}  

#[derive(Debug, Default, Clone)]
/// Defines a parallel composition --- notes that are on top of each other (chords, etc)
struct Parallel {
    elements: Vec<Box<MusicElement>>,
}

impl MusicElement for Parallel {
    // Due to all of the notes appearing in parallel, the overall duration must be that of the longest one
    fn duration(&self) -> f32 {
        self.elements.iter().fold(std::f32::NEG_INFINITY, |acc, ref e| e.duration().max(acc))
    }

    fn set_channel(&mut self, channel: Channel) {
        for element in self.elements.iter_mut() {
            element.set_channel(channel.clone());
        }
    }
}

/// Helper macro for parallel compositions, packs the inputs in a box before moving them into the vector
macro_rules! parallel {
    ($($e:expr),*) => ({
        let mut seq = Parallel::default();
        $(
            seq.elements.push(Box::new($e));
        )*
        seq
    })
}  

fn main() {
    // defines helper functions for creating notes
    let c4 = |dur| Note::new(NoteClass::C, 4).duration(dur);
    let d4 = |dur| Note::new(NoteClass::D, 4).duration(dur);
    let e4 = |dur| Note::new(NoteClass::E, 4).duration(dur);
    let f4 = |dur| Note::new(NoteClass::F, 4).duration(dur);
    let g4 = |dur| Note::new(NoteClass::G, 4).duration(dur);
    let c5 = |dur| Note::new(NoteClass::C, 5).duration(dur);

    // defines the triplets used in the song
    let c5_triplet = sequence![c5(1.0/12.0), c5(1.0/12.0), c5(1.0/12.0)];
    let g4_triplet = sequence![g4(1.0/12.0), g4(1.0/12.0), g4(1.0/12.0)];
    let e4_triplet = sequence![e4(1.0/12.0), e4(1.0/12.0), e4(1.0/12.0)];
    let c4_triplet = sequence![c4(1.0/12.0), c4(1.0/12.0), c4(1.0/12.0)];

    // row row row your boat
    let music = sequence![c4(0.25), c4(0.25), c4(3.0/16.0), d4(1.0/16.0), e4(0.25),
                          e4(3.0/16.0), d4(1.0/16.0), e4(3.0/16.0), f4(1.0/16.0), g4(0.5),
                          c5_triplet, g4_triplet, e4_triplet, c4_triplet,
                          g4(3.0/16.0), f4(1.0/16.0), e4(3.0/16.0), d4(1.0/16.0), c4(0.5)];

    // change the channel to organ and violin for the offset parts of the canon
    let mut music_organ  = music.clone();
    let mut music_violin = music.clone();
    music_organ.set_channel(Channel::Organ);
    music_violin.set_channel(Channel::Violin);

    println!("Length of music: {} bars", music.duration() / MIDI_TEMPO);

    let canon = parallel![music, sequence![Pause::new(1.0), music_organ], sequence![Pause::new(2.0), music_violin]];
    println!("Length of canon: {} bars", canon.duration() / MIDI_TEMPO);
    //println!("Full canon: {:#?}", canon);
}
