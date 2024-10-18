use rosbag::RosBag;

mod messages;
mod roswrap;
use byteorder::WriteBytesExt;
use std::{fs, io::Write};

fn parse_args() -> (String, String) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <bag-file> <output>", args[0]);
        std::process::exit(1);
    }
    (args[1].clone(), args[2].clone())
}

const MAX_EVENTS_PER_MESSAGE: u32 = 800_000;

fn main() {
    let (bag_filename, output_file) = parse_args();

    let bag = RosBag::new(&bag_filename).unwrap();

    let mut event_buffer =
        vec![0; (MAX_EVENTS_PER_MESSAGE * messages::SERIALIZED_EVENT_SIZE) as usize];

    let chunks = roswrap::chunk_iter(&bag);
    let mut i = 0;

    let mut output = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(output_file.clone())
        .unwrap();

    let mut width: u32 = 0;
    let mut height: u32 = 0;

    let mut total_events = 0;
    for chunk in chunks {
        let messages = roswrap::msg_iter(&chunk);
        for msg_data in messages {
            if width == 0 {
                let meta = roswrap::read_event_array_from_bytes(&msg_data.data, true);
                width = meta.width;
                height = meta.height;
            }
            let used_event_bytes = roswrap::read_events_from_raw_events_array_msg_to_buffer(
                &msg_data.data,
                &mut event_buffer,
                0,
            );

            total_events += used_event_bytes / messages::SERIALIZED_EVENT_SIZE;

            output
                .write_all(&event_buffer[..used_event_bytes as usize])
                .unwrap();

            if i % 1000 == 0 {
                println!("Processed {} messages", i);
            }

            i += 1;
        }
    }

    println!("Done! Parsed events: {}", total_events);
    output
        .write_u16::<byteorder::LittleEndian>(width as u16)
        .unwrap();
    output
        .write_u16::<byteorder::LittleEndian>(height as u16)
        .unwrap();
    output
        .write_u32::<byteorder::LittleEndian>(total_events)
        .unwrap();

    println!("Data written to {}", output_file);
}
