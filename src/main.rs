extern crate ffmpeg_next as ffmpeg;
use ffmpeg::media::Type;
use rosbag::RosBag;

mod export;
mod ffmpegwrap;
mod messages;
mod roswrap;

const HEIGHT: u32 = 480;
const WIDTH: u32 = 640;
const MAX_EVENTS_PER_FRAME: usize = 10_000_000;

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

    let mut ictx = ffmpeg::format::input(&mp4_filename).unwrap();
    let input = ictx.streams().best(Type::Video).unwrap();
    let video_stream_index = input.index();
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

    let bag = RosBag::new(&bag_filename).unwrap();
    let mut frame = ffmpeg::util::frame::Video::empty();
    let mut frame_rgb = ffmpeg::util::frame::Video::empty();
    let mut event_buffer = vec![0; MAX_EVENTS_PER_FRAME * messages::SERIALIZED_EVENT_SIZE as usize];
    let mut used_event_bytes: u32 = 0;
    let mut frame_idx = 0;
    let mut initial_event_ts = 0;
    let mut packet_iter = ictx.packets();

    let mut frame_ts = ffmpegwrap::parse_next_frame(
        video_stream_index as i32,
        &mut decoder,
        &mut scaler,
        &mut packet_iter,
        &mut frame,
        &mut frame_rgb,
    )
    .unwrap();

    let mut total_time_parsing_rosbag = 0;
    let mut total_time_parsing_video = 0;
    let mut total_time_exporting = 0;

    let mut done: bool = false;

    let mut i = 0;
    let chunks = roswrap::chunk_iter(&bag);

    for chunk in chunks {
        let messages = roswrap::msg_iter(&chunk);
        for msg_data in messages {
            let now = std::time::Instant::now();
            let read_res = roswrap::read_events_from_raw_events_array_msg_to_buffer(
                &msg_data.data,
                &mut event_buffer,
                used_event_bytes,
            );
            used_event_bytes = read_res.0;
            let mut event_ts = read_res.1;

            if initial_event_ts == 0 {
                initial_event_ts = event_ts;
            }
            event_ts -= initial_event_ts;
            total_time_parsing_rosbag += now.elapsed().as_micros();
            if event_ts > frame_ts {
                // TODO: analyze if using message timestamp is sufficient

                let now = std::time::Instant::now();
                let frame_idx_padded = format!("{:05}", frame_idx);
                let output = format!("{}_{}.bin", output, frame_idx_padded);
                export::export_frame_with_events(
                    &frame_rgb,
                    &event_buffer,
                    &output,
                    used_event_bytes,
                );
                total_time_exporting += now.elapsed().as_micros();

                let now = std::time::Instant::now();
                let parse_res = ffmpegwrap::parse_next_frame(
                    video_stream_index as i32,
                    &mut decoder,
                    &mut scaler,
                    &mut packet_iter,
                    &mut frame,
                    &mut frame_rgb,
                );
                total_time_parsing_video += now.elapsed().as_micros();
                if parse_res.is_some() {
                    frame_ts = parse_res.unwrap();
                    used_event_bytes = 0;
                    frame_idx += 1;
                } else {
                    done = true;
                    break;
                }
            }

            if i % 1000 == 0 {
                println!("Processed {} messages", i);
            }
            i += 1;
        }
        if done {
            break;
        }
    }
    println!(
        "Total time parsing rosbag: {} ms",
        total_time_parsing_rosbag as f64 / 1e3
    );
    println!(
        "Total time parsing video: {} ms",
        total_time_parsing_video as f64 / 1e3
    );
    println!(
        "Total time exporting: {} ms",
        total_time_exporting as f64 / 1e3
    );
}
