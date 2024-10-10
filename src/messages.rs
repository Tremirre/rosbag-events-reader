#[derive(Debug, Clone)]
pub struct Time {
    pub sec: i32,
    pub nsec: u32,
}

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
        let res = ((self.sec as f32) * 1e3 + (self.nsec as f32) * 1e-6);
        return res as u64;
    }
}
