extern crate ffmpeg_next as ffmpeg;
use ffmpeg::media::Type;
use rosbag::RosBag;
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
struct Timestamp {
    raw: i64,
    seconds: f64,
    milliseconds: i64,
    timebase: ffmpeg::Rational,
}

impl Timestamp {
    fn from_frame(
        frame: &ffmpeg::util::frame::Video,
        stream: &ffmpeg_next::Stream,
    ) -> Option<Self> {
        frame.timestamp().map(|ts| {
            let timebase = stream.time_base();
            let seconds =
                ts as f64 * f64::from(timebase.numerator()) / f64::from(timebase.denominator());
            let milliseconds = (seconds * 1000.0) as i64;

            Self {
                raw: ts,
                seconds,
                milliseconds,
                timebase,
            }
        })
    }
}
fn frame_to_buffer<'a>(frame: &'a ffmpeg::util::frame::Video) -> Vec<u8> {
    let width = frame.width() as usize;
    let height = frame.height() as usize;
    let stride = frame.stride(0);
    let mut frame_data = vec![0; width * height * 3];
    for y in 0..height {
        for x in 0..width {
            let src_idx = y * stride + x * 3;
            let dst_idx = y * width * 3 + x * 3;
            frame_data[dst_idx] = frame.data(0)[src_idx];
            frame_data[dst_idx + 1] = frame.data(0)[src_idx + 1];
            frame_data[dst_idx + 2] = frame.data(0)[src_idx + 2];
        }
    }
    frame_data
}

fn parse_args() -> (String, String, String) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: {} <bag-file> <mp4-file> <output>", args[0]);
        std::process::exit(1);
    }
    (args[1].clone(), args[2].clone(), args[3].clone())
}

fn main() {
    let (bag_filename, mp4_filename, output) = parse_args();

    match ffmpeg::init() {
        Ok(_) => println!("FFmpeg initialized"),
        Err(e) => eprintln!("Error initializing ffmpeg: {}", e),
    }

    // ==== FRAMES ===
    let mut ictx = ffmpeg::format::input(&mp4_filename).unwrap();
    let input = ictx.streams().best(Type::Video).unwrap();
    let vide_stream_index = input.index();
    let ctx_decoder = ffmpeg::codec::context::Context::from_parameters(input.parameters()).unwrap();
    let mut decoder = ctx_decoder.decoder().video().unwrap();
    let mut scaler = ffmpeg::software::scaling::context::Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        ffmpeg::format::Pixel::RGB24,
        WIDTH,
        HEIGHT,
        ffmpeg::software::scaling::flag::Flags::BILINEAR,
    )
    .unwrap();

    // ==== EVENTS ===
    let bag = RosBag::new(&bag_filename).unwrap();
    let mut frame = ffmpeg::util::frame::Video::empty();
    let mut frame_rgb = ffmpeg::util::frame::Video::empty();
    let mut events = Vec::<messages::Event>::new();

    let mut frame_idx = 0;

    for (stream, packet) in ictx.packets() {
        if stream.index() == vide_stream_index {
            let _ = decoder.send_packet(&packet);
            let decoded = decoder.receive_frame(&mut frame);
            if decoded.is_err() {
                continue;
            }

            let ts = Timestamp::from_frame(&frame, &stream).unwrap();
            println!("Frame {}: {}", frame_idx, ts.milliseconds);
            let _ = scaler.run(&frame, &mut frame_rgb);
            let frame_rgb = &frame_rgb;
            let output_file = format!("{}/frame_{}.rgb", output, frame_idx);
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(&output_file)
                .unwrap();

            let frame_data = frame_to_buffer(&frame_rgb);
            file.write_all(&frame_data).unwrap();

            frame_idx += 1;
            // if frame_idx % 100 == 0 {
            //     println!("Processed {} frames", frame_idx);
            // }
        }
    }
    let _ = decoder.send_eof();

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
