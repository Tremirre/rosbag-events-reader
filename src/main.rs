use rosbag::{ChunkRecord, MessageRecord, MessageRecordsIterator, RosBag};
use std::{fs, io::Write};

mod messages;
mod roswrap;

const HEIGHT: u32 = 480;
const WIDTH: u32 = 640;

// TODO: Create a new pipeline that:
// 1. Reads the mp4 file
// 2. Extracts the frames, scales them down to 640x480
// 3. Extracts the timestamps
// 4. Reads the ros bag file
// 5. Extracts the events as per the frame timestamps
// 6. Writes the events and frames to a binary file

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <bag-file> <mp4-file> <output>", args[0]);
        std::process::exit(1);
    }
    let bag_filename = &args[1];

    let bag = RosBag::new(&bag_filename).unwrap();
    let msg_count = bag
        .chunk_records()
        .map(|r| {
            if let Ok(ChunkRecord::Chunk(chunk)) = r {
                chunk.messages().count()
            } else {
                0
            }
        })
        .sum::<usize>();

    println!("Total messages: {}", msg_count);

    let mut events = Vec::<messages::Event>::new();

    println!("Reading messages");

    // use the iterator to get the events
    let mut i = 0;
    let chunks = roswrap::chunk_iter(&bag);
    chunks.for_each(|chunk| {
        let messages = roswrap::msg_iter(&chunk);
        messages.for_each(|msg_data| {
            let event_data = roswrap::read_event_array_from_bytes(msg_data.data);
            events.extend(event_data.events);

            if i % 1000 == 0 {
                println!("Processed {} messages", i);
            }
            i += 1;
        });
    });
    println!("Total events: {}", events.len());
    // println!("Writing frames to binary file: {}", output);
    // let mut file = fs::OpenOptions::new()
    //     .write(true)
    //     .create(true)
    //     .open(output)
    //     .unwrap();
    // file.write_u32::<byteorder::LittleEndian>(HEIGHT as u32)
    //     .unwrap();
    // file.write_u32::<byteorder::LittleEndian>(WIDTH as u32)
    //     .unwrap();
    // file.write_u32::<byteorder::LittleEndian>(msg_count as u32)
    //     .unwrap();
    // file.write_all(&frame_buffer).unwrap();
    // let _ = file.write_all(unsafe {
    //     std::slice::from_raw_parts(
    //         timestamps.as_ptr() as *const u8,
    //         timestamps.len() * std::mem::size_of::<f32>(),
    //     )
    // });
}
