#[derive(Debug, Clone)]
pub struct Time {
    pub sec: i32,
    pub nsec: u32,
}

pub const SERIALIZED_EVENT_SIZE: u32 = 8;

#[derive(Debug)]
pub struct Event {
    pub x: u16,
    pub y: u16,
    pub ts: Time,
    pub polarity: bool,
}

#[derive(Debug)]
pub struct Header {
    pub seq: u32,
    pub stamp: Time,
    pub frame_id: String,
}

#[derive(Debug)]
pub struct EventArray {
    pub header: Header,
    pub height: u32,
    pub width: u32,
    pub events: Vec<Event>,
}

impl Time {
    pub fn msec(&self) -> u64 {
        let res = (self.sec as f32) * 1e3 + (self.nsec as f32) * 1e-6;
        return res as u64;
    }

    pub fn microsec(&self) -> u64 {
        let res = (self.sec as f32) * 1e6 + (self.nsec as f32) * 1e-3;
        return res as u64;
    }
}

impl Event {
    pub fn to_buffer(&self, buffer: &mut Vec<u8>, buffer_offset: u32) -> u32 {
        let indexable_offset = buffer_offset as usize;
        let msec = self.ts.msec();
        assert!(buffer.len() >= indexable_offset + SERIALIZED_EVENT_SIZE as usize);
        assert!(msec < 1 << 24);
        buffer[indexable_offset] = (msec & 0xFF) as u8;
        buffer[indexable_offset + 1] = (msec >> 8) as u8;
        buffer[indexable_offset + 2] = (msec >> 16) as u8;
        buffer[indexable_offset + 3] = (self.x & 0xFF) as u8;
        buffer[indexable_offset + 4] = (self.x >> 8) as u8;
        buffer[indexable_offset + 5] = (self.y & 0xFF) as u8;
        buffer[indexable_offset + 6] = (self.y >> 8) as u8;
        buffer[indexable_offset + 7] = if self.polarity { 1 } else { 0 };
        buffer_offset + SERIALIZED_EVENT_SIZE
    }
}
