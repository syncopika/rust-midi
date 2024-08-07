// https://github.com/insomnimus/midnote

use clap::{arg, command, ArgAction};//, value_parser, ArgAction, Command};
use std::{path::Path, error::Error};
use midly::Smf;
use std::collections::HashMap;
use nodi;

struct MidiInfo {
    num_tracks: usize,
    ports: HashMap<usize, Vec<u8>>,
    channels: HashMap<usize, Vec<u8>>,
    instruments: HashMap<usize, Vec<String>>, // separate by track
    tempi: Vec<f32>, // Vec in case there are tempo changes?
    notes_per_track: HashMap<usize, usize>,
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
    instruments_map.insert(24, String::from("guitar"));
    
    let mut midi_info = MidiInfo {
        num_tracks: tracks.len(),
        ports: HashMap::new(),
        channels: HashMap::new(),
        instruments: HashMap::new(),
        tempi: Vec::new(),
        notes_per_track: HashMap::new(),
    };
    
    for (i, track) in tracks.iter().enumerate() {
        //println!("track event length: {}", track.len());
        
        let mut track_ports = Vec::new();
        let mut track_channels = Vec::new();
        let mut track_instruments = Vec::new();
        let mut num_notes = 0;
        
        for track_event in track.iter() {
            match track_event.kind {
                midly::TrackEventKind::Midi{channel: _, message} => {
                    //println!("got a midi event");
                    match message {
                        midly::MidiMessage::ProgramChange{program} => {
                            track_instruments.push(
                                instruments_map
                                  .get(&program.as_int())
                                  .unwrap_or(&program.to_string())
                                  .to_string()
                            );
                        },
                        midly::MidiMessage::NoteOn{key: _, vel} => {
                            if vel != 0 {
                                // avoid double-counting notes because velocity could be 0,
                                // which is essentially a NoteOff-equivalent message
                                num_notes += 1;
                            }
                        },
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
                        midly::MetaMessage::MidiChannel(channel) => {
                            //println!("got midi channel: {}", channel);
                            track_channels.push(channel.as_int());
                        },
                        midly::MetaMessage::MidiPort(port) => {
                            //println!("got midi port: {}", port);
                            track_ports.push(port.as_int());
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
        
        midi_info.ports.insert(i+1, track_ports);
        midi_info.channels.insert(i+1, track_channels);
        midi_info.instruments.insert(i+1, track_instruments);
        midi_info.notes_per_track.insert(i+1, num_notes);
    }
    
    return midi_info;
}

// https://github.com/insomnimus/nodi/blob/main/examples/play_midi.rs#L69
fn get_connection(n: usize) -> Result<midir::MidiOutputConnection, Box<dyn Error>> {
    let midi_out = midir::MidiOutput::new("play_midi")?;

    let out_ports = midi_out.ports();
    
    if out_ports.is_empty() {
      return Err("no MIDI output device detected".into());
    }
    
    if n >= out_ports.len() {
      return Err(format!(
        "only {} MIDI devices detected",
        out_ports.len()
      ).into());
    }
    
    println!("number of MIDI output ports available: {}", out_ports.len());

    let out_port = &out_ports[n];
    
    let port_name = midi_out.port_name(out_port).unwrap();
    println!("port name: {}", port_name);
    
    let out = midi_out.connect(out_port, "port-name")?;
    Ok(out)
}

fn display_midi_info(midi_info: MidiInfo) {
    println!("the file has {} tracks", midi_info.num_tracks);
    //println!("num instruments: {}", midi_info.instruments.len());
    println!("num tempi: {}", midi_info.tempi.len());
    println!("tempi: {:?}", midi_info.tempi);
    
    let mut track_nums: Vec<&usize> = midi_info.instruments.keys().collect();
    track_nums.sort();
    
    for track in track_nums {
        println!("track {} instruments: {:?}", track, midi_info.instruments.get(track).unwrap());
        println!("track {} channels: {:?}", track, midi_info.channels.get(track).unwrap());
        println!("track {} ports: {:?}", track, midi_info.ports.get(track).unwrap());
        println!("track {} number of notes: {:?}", track, midi_info.notes_per_track.get(track).unwrap());
        println!("--------------------------");
    }
}

// https://github.com/insomnimus/nodi/blob/main/examples/play_midi.rs#L45
fn play(smf: &midly::Smf) -> Result<(), Box<dyn Error>> {
    let timer = nodi::timers::Ticker::try_from(smf.header.timing)?;

    let conn = get_connection(0)?; // TODO: don't hardcode device number

    let sheet = match smf.header.format {
        midly::Format::SingleTrack | midly::Format::Sequential => nodi::Sheet::sequential(&smf.tracks),
        midly::Format::Parallel => nodi::Sheet::parallel(&smf.tracks),
    };

    let mut player = nodi::Player::new(timer, conn);

    println!("starting playback");
    player.play_sheet(&sheet);
    
    Ok(())
}

fn main() {
    let matches = command!()
        .arg(
            arg!(-f --filepath <VALUE>)
                .required(true)
                .help("path to the MIDI file")
         )
        .arg(
            arg!(-p --play)
                .action(ArgAction::SetTrue)
                .help("play the MIDI file")
         )
        .get_matches();
        
    if let Some(filepath) = matches.get_one::<String>("filepath") {
        println!("MIDI filepath: {filepath}");
        
        // make sure file exists
        //assert_eq!(Path::new(filepath).try_exists().is_ok(), true); // this doesn't seem to be working?? :/
        assert_eq!(Path::new(filepath).exists(), true);
        
        let data = std::fs::read(filepath).unwrap();
        let smf = Smf::parse(&data).unwrap();
        
        let midi_info = get_midi_info(smf.tracks);
        display_midi_info(midi_info);
        
        if *matches.get_one::<bool>("play").unwrap() {
            // play the file
            let smf2 = Smf::parse(&data).unwrap();
            let _ = play(&smf2);
        }
    }
}
