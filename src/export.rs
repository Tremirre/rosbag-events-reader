extern crate ffmpeg_next as ffmpeg;

use crate::messages;
use byteorder::WriteBytesExt;
use ffmpeg::util::frame::Video;
use std::{fs, io::Write};

pub fn export_frame_with_events(
    frame_rgb: &Video,
    events_buffer: &Vec<u8>,
    output_path: &str,
    used_event_bytes: u32,
) {
    let mut output = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(output_path)
        .unwrap();

    output
        .write_u32::<byteorder::LittleEndian>(frame_rgb.width() as u32)
        .unwrap();
    output
        .write_u32::<byteorder::LittleEndian>(frame_rgb.height() as u32)
        .unwrap();
    output
        .write_u32::<byteorder::LittleEndian>(used_event_bytes / messages::SERIALIZED_EVENT_SIZE)
        .unwrap();

    output.write_all(frame_rgb.data(0)).unwrap();
    output
        .write_all(&events_buffer[..used_event_bytes as usize])
        .unwrap();
}
