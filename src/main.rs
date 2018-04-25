#![allow(dead_code, unused_macros)]

// imports
use std::fmt;

// constants
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

trait MusicElement : fmt::Debug + Cloneable {
    fn duration(&self) -> f32;
    fn set_channel(&mut self, channel: Channel);
}

impl Clone for Box<MusicElement> {
    fn clone(&self) -> Box<MusicElement> {
        self.clone_box()
    }
}

// enums
#[derive(Debug, Clone)]
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
enum Channel {
    Piano, Organ, Guitar, Violin, Flute, Trumpet, Helicopter, Telephone
}

impl Channel {
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
struct Note {
    name: NoteClass,
    octave: u32,
    instrument: Channel,
    duration: f32,
}

impl Note {
    fn new(name: NoteClass, octave: u32) -> Note {
        Note { name, octave, instrument: Channel::Piano, duration: 0.25 }
    }
    
    fn channel(mut self, instrument: Channel) -> Note {
        self.instrument = instrument;
        self
    }
    
    fn duration(mut self, duration: f32) -> Note {
        self.duration = duration;
        self
    }
    
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
struct Pause {
    duration: f32,
}

impl Pause {
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
    let c4 = |dur| Note::new(NoteClass::C, 4).duration(dur);
    let d4 = |dur| Note::new(NoteClass::D, 4).duration(dur);
    let e4 = |dur| Note::new(NoteClass::E, 4).duration(dur);
    let f4 = |dur| Note::new(NoteClass::F, 4).duration(dur);
    let g4 = |dur| Note::new(NoteClass::G, 4).duration(dur);
    let c5 = |dur| Note::new(NoteClass::C, 5).duration(dur);

    let c5_triplet = sequence![c5(1.0/12.0), c5(1.0/12.0), c5(1.0/12.0)];
    let g4_triplet = sequence![g4(1.0/12.0), g4(1.0/12.0), g4(1.0/12.0)];
    let e4_triplet = sequence![e4(1.0/12.0), e4(1.0/12.0), e4(1.0/12.0)];
    let c4_triplet = sequence![c4(1.0/12.0), c4(1.0/12.0), c4(1.0/12.0)];

    // row row row your boat
    let music = sequence![c4(0.25), c4(0.25), c4(3.0/16.0), d4(1.0/16.0), e4(0.25),
                          e4(3.0/16.0), d4(1.0/16.0), e4(3.0/16.0), f4(1.0/16.0), g4(0.5),
                          c5_triplet, g4_triplet, e4_triplet, c4_triplet,
                          g4(3.0/16.0), f4(1.0/16.0), e4(3.0/16.0), d4(1.0/16.0), c4(0.5)];

    let mut music_organ  = music.clone();
    let mut music_violin = music.clone();
    music_organ.set_channel(Channel::Organ);
    music_violin.set_channel(Channel::Violin);

    println!("Length of music: {} bars", music.duration() / MIDI_TEMPO);

    let canon = parallel![music, sequence![Pause::new(1.0), music_organ], sequence![Pause::new(2.0), music_violin]];
    println!("Length of canon: {} bars", canon.duration() / MIDI_TEMPO);
    //println!("Full canon: {:#?}", canon);
}
