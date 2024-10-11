extern crate ffmpeg_next as ffmpeg;

pub fn read_timestamp_ms_from_frame(
    frame: &ffmpeg::util::frame::Video,
    stream: &ffmpeg::Stream,
) -> u64 {
    let timebase = stream.time_base();
    let multiplier = f64::from(timebase.numerator()) / f64::from(timebase.denominator());
    let seconds = (frame.timestamp().unwrap() as f64) * multiplier;
    (seconds * 1000.0) as u64
}

pub fn parse_next_frame(
    stream_idx: i32,
    decoder: &mut ffmpeg::codec::decoder::Video,
    scaler: &mut ffmpeg::software::scaling::context::Context,
    packet_iter: &mut ffmpeg::format::context::input::PacketIter,
    frame: &mut ffmpeg::util::frame::Video,
    frame_rgb: &mut ffmpeg::util::frame::Video,
) -> Option<u64> {
    let res = packet_iter.next();
    if res.is_none() {
        return None;
    }
    let (stream, packet) = res.unwrap();
    if stream.index() != stream_idx as usize {
        return None;
    }

    let send_res = decoder.send_packet(&packet);
    if send_res.is_err() {
        return None;
    }
    let rec_res = decoder.receive_frame(frame);
    if rec_res.is_err() {
        return None;
    }
    let sc_res = scaler.run(&frame, frame_rgb);
    if sc_res.is_err() {
        return None;
    }

    Some(read_timestamp_ms_from_frame(frame, &stream))
}
