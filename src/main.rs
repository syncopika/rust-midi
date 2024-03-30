// https://github.com/insomnimus/midnote

use clap::{arg, command};//, value_parser, ArgAction, Command};
use std::path::Path;
use midly::Smf;
use std::collections::HashMap;

struct MidiInfo {
    num_tracks: usize,
    instruments: HashMap<usize, Vec<String>>, // separate by track
    tempi: Vec<f32>, // Vec in case there are tempo changes?
}

fn get_midi_info(tracks: Vec<Vec<midly::TrackEvent>>) -> MidiInfo {
    // https://www.recordingblogs.com/wiki/midi-program-change-message
    let mut instruments_map = HashMap::new();
    instruments_map.insert(40, String::from("violin"));
    instruments_map.insert(41, String::from("viola"));
    instruments_map.insert(42, String::from("cello"));
    instruments_map.insert(43, String::from("contrabass"));
    instruments_map.insert(44, String::from("tremolo strings"));
    instruments_map.insert(45, String::from("pizzicato strings"));
    instruments_map.insert(46, String::from("harp"));
    instruments_map.insert(47, String::from("timpani"));
    instruments_map.insert(48, String::from("string ensemble 1"));
    instruments_map.insert(56, String::from("trumpet"));
    instruments_map.insert(57, String::from("trombone"));
    instruments_map.insert(58, String::from("tuba"));
    instruments_map.insert(60, String::from("french horn"));
    instruments_map.insert(68, String::from("oboe"));
    instruments_map.insert(69, String::from("english horn"));
    instruments_map.insert(70, String::from("bassoon"));
    instruments_map.insert(71, String::from("clarinet"));
    instruments_map.insert(72, String::from("piccolo"));
    instruments_map.insert(73, String::from("flute"));
    instruments_map.insert(74, String::from("recorder"));
    instruments_map.insert(9, String::from("glockenspiel"));
    instruments_map.insert(14, String::from("tubular bell"));
    instruments_map.insert(12, String::from("marimba"));
    instruments_map.insert(11, String::from("vibraphone"));
    instruments_map.insert(0, String::from("acoustic grand piano"));
    
    let mut midi_info = MidiInfo {
        num_tracks: tracks.len(),
        instruments: HashMap::new(),
        tempi: Vec::new(),
    };
    
    for (i, track) in tracks.iter().enumerate() {
        //println!("track event length: {}", track.len());
        let mut track_instruments = Vec::new();
        
        for track_event in track.iter() {
            match track_event.kind {
                midly::TrackEventKind::Midi{channel, message} => {
                    //println!("got a midi event");
                    match message {
                        midly::MidiMessage::ProgramChange{program} => {
                            track_instruments.push(
                                instruments_map
                                  .get(&program.as_int())
                                  .unwrap_or(&program.to_string())
                                  .to_string()
                            );
                        }
                        _ => (),
                    }
                },
                midly::TrackEventKind::Meta(msg) => {
                    //println!("got a metamessage");
                    match msg {
                        midly::MetaMessage::TrackName(_track_name) => {
                            //println!("got track name");
                        },
                        midly::MetaMessage::InstrumentName(_inst_name) => {
                            //println!("got instrument name");
                        },
                        midly::MetaMessage::TrackNumber(_track_num) => {
                            //println!("got track num");
                        },
                        midly::MetaMessage::MidiChannel(_channel) => {
                            //println!("got midi channel");
                        },
                        midly::MetaMessage::MidiPort(_port) => {
                            //println!("got midi port");
                        },
                        midly::MetaMessage::Tempo(tempo) => {
                            // we get tempo as microseconds per beat and there's 6e7 microseconds in a minute
                            //println!("got tempo (bpm): {}", 60000000.0 / (tempo.as_int() as f32));
                            midi_info.tempi.push(60000000.0 / (tempo.as_int() as f32));
                        },
                        _ => (),
                    }
                },
                _ => (),
            }
        }
        
        midi_info.instruments.insert(i+1, track_instruments);
    }
    
    return midi_info;
}

fn main() {
    // TODO: add flag to get MIDI info like instruments, num tracks, etc.
    // TODO: add flag to play MIDI file
    let matches = command!()
        .arg(arg!(--filepath <VALUE>).required(true))
        .get_matches();
        
    if let Some(filepath) = matches.get_one::<String>("filepath") {
        println!("MIDI filepath: {filepath}");
        
        // make sure file exists
        //assert_eq!(Path::new(filepath).try_exists().is_ok(), true); // this doesn't seem to be working?? :/
        assert_eq!(Path::new(filepath).exists(), true);
        
        let data = std::fs::read(filepath).unwrap();
        let smf = Smf::parse(&data).unwrap();
        
        //println!("------------------");
        
        let midi_info = get_midi_info(smf.tracks);
        
        println!("the file has {} tracks", midi_info.num_tracks);
        //println!("num instruments: {}", midi_info.instruments.len());
        println!("num tempi: {}", midi_info.tempi.len());
        
        println!("instruments per track:");
        for (track, instruments) in midi_info.instruments {
            println!("track: {}: {:?}", track, instruments);
        }
        
        println!("tempi: {:?}", midi_info.tempi);
    }
}
