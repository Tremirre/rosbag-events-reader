use crate::messages;
use byteorder::ReadBytesExt;
use rosbag::record_types::{Chunk, MessageData};
use rosbag::{ChunkRecord, MessageRecord, RosBag};

pub fn read_event_array_from_bytes(data: &[u8], meta_only: bool) -> messages::EventArray {
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
    let capacity = if meta_only { 0 } else { num_events as usize };
    let mut events = Vec::with_capacity(capacity);
    if meta_only {
        return messages::EventArray {
            header,
            height,
            width,
            events,
        };
    }
    for _ in 0..num_events {
        let x = width as u16 - cursor.read_u16::<byteorder::LittleEndian>().unwrap() - 1;
        let y = height as u16 - cursor.read_u16::<byteorder::LittleEndian>().unwrap() - 1;
        events.push(messages::Event {
            x,
            y,
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

pub fn read_events_from_raw_events_array_msg_to_buffer(
    data: &[u8],
    events_buffer: &mut Vec<u8>,
    used_event_bytes: u32,
) -> u32 {
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
    let mut offset = used_event_bytes;
    for _ in 0..num_events {
        let x = width as u16 - cursor.read_u16::<byteorder::LittleEndian>().unwrap() - 1;
        let y = height as u16 - cursor.read_u16::<byteorder::LittleEndian>().unwrap() - 1;
        let event = messages::Event {
            x,
            y,
            ts: messages::Time {
                sec: cursor.read_i32::<byteorder::LittleEndian>().unwrap(),
                nsec: cursor.read_u32::<byteorder::LittleEndian>().unwrap(),
            },
            polarity: cursor.read_u8().unwrap() != 0,
        };
        offset = event.to_buffer(events_buffer, offset);
    }
    offset
}

pub fn chunk_iter(bag: &RosBag) -> impl Iterator<Item = Chunk> {
    bag.chunk_records().filter_map(|r| {
        if let Ok(ChunkRecord::Chunk(chunk)) = r {
            Some(chunk)
        } else {
            None
        }
    })
}

pub fn msg_iter<'a>(chunk: &'a Chunk) -> impl Iterator<Item = MessageData<'a>> {
    chunk.messages().filter_map(|m| {
        if let Ok(MessageRecord::MessageData(msg_data)) = m {
            Some(msg_data)
        } else {
            None
        }
    })
}
