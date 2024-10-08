use byteorder::{ReadBytesExt, WriteBytesExt};
use rosbag::{ChunkRecord, MessageRecord, RosBag};
use std::{fs, io::Write};

mod messages;

fn read_event_array_from_bytes(data: &[u8]) -> messages::EventArray {
    let mut cursor = std::io::Cursor::new(data);
    let header = messages::Header {
        seq: cursor.read_u32::<byteorder::LittleEndian>().unwrap(),
        stamp: messages::Time {
            sec: cursor.read_i32::<byteorder::LittleEndian>().unwrap(),
            nsec: cursor.read_u32::<byteorder::LittleEndian>().unwrap(),
        },
        frame_id: {
            let len = cursor.read_u32::<byteorder::LittleEndian>().unwrap();
            let mut buf = vec![0; len as usize];
            for i in 0..len {
                buf[i as usize] = cursor.read_u8().unwrap();
            }
            String::from_utf8(buf).unwrap()
        },
    };
    let height = cursor.read_u32::<byteorder::LittleEndian>().unwrap();
    let width = cursor.read_u32::<byteorder::LittleEndian>().unwrap();
    let num_events = cursor.read_u32::<byteorder::LittleEndian>().unwrap();
    let mut events = Vec::with_capacity(num_events as usize);
    for _ in 0..num_events {
        events.push(messages::Event {
            x: cursor.read_u16::<byteorder::LittleEndian>().unwrap(),
            y: cursor.read_u16::<byteorder::LittleEndian>().unwrap(),
            ts: messages::Time {
                sec: cursor.read_i32::<byteorder::LittleEndian>().unwrap(),
                nsec: cursor.read_u32::<byteorder::LittleEndian>().unwrap(),
            },
            polarity: cursor.read_u8().unwrap() != 0,
        });
    }
    messages::EventArray {
        header,
        height,
        width,
        events,
    }
}

fn events_to_buffer(
    events: &Vec<messages::Event>,
    frame_buffer: &mut Vec<u8>,
    width: u32,
    height: u32,
    frame_index: usize,
) {
    for event in events {
        let x = (WIDTH as u16 - event.x) as usize;
        let y = (HEIGHT as u16 - event.y) as usize;
        let polarity = if event.polarity { 255 } else { 0 };
        let index = y * width as usize + x;
        frame_buffer[frame_index * width as usize * height as usize + index] = polarity;
    }
}

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
    if args.len() < 2 {
        eprintln!("Usage: {} <bagfile> <output>", args[0]);
        std::process::exit(1);
    }
    let filename = &args[1];

    let mut output = "exported";
    if args.len() > 2 {
        output = &args[2];
    }

    let bag = RosBag::new(&filename).unwrap();
    let mut i = 0;
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

    let mut frame_buffer: Vec<u8> = vec![127; WIDTH as usize * HEIGHT as usize * msg_count];
    let mut timestamps: Vec<f32> = vec![0.0; msg_count];

    println!("Reading messages");
    for record in bag.chunk_records() {
        if let Ok(ChunkRecord::Chunk(chunk)) = record {
            for msg in chunk.messages() {
                if let Ok(MessageRecord::MessageData(msg_data)) = msg {
                    let event_data = read_event_array_from_bytes(msg_data.data);
                    events_to_buffer(&event_data.events, &mut frame_buffer, WIDTH, HEIGHT, i);
                    let timestamp = event_data.header.stamp.sec as f32
                        + event_data.header.stamp.nsec as f32 / 1e9;
                    timestamps[i] = timestamp;
                    i += 1;
                    if i % 1000 == 0 {
                        println!("Message #{}/{}", i, msg_count);
                    }
                }
            }
        }
    }
    println!("Writing frames to binary file: {}", output);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(output)
        .unwrap();
    file.write_u32::<byteorder::LittleEndian>(HEIGHT as u32)
        .unwrap();
    file.write_u32::<byteorder::LittleEndian>(WIDTH as u32)
        .unwrap();
    file.write_u32::<byteorder::LittleEndian>(msg_count as u32)
        .unwrap();
    file.write_all(&frame_buffer).unwrap();
    let _ = file.write_all(unsafe {
        std::slice::from_raw_parts(
            timestamps.as_ptr() as *const u8,
            timestamps.len() * std::mem::size_of::<f32>(),
        )
    });
}
