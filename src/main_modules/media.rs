use serenity::all::{Attachment, EditMessage, Message};
use uuid::Uuid;
use std::io::Write;
use std::process::{Command, Output};
use image::{GenericImageView, ImageBuffer, Rgba, DynamicImage};
use std::path::{Path, PathBuf};
use super::REQWEST_CLIENT;
use std::fs;
use rayon::prelude::*;
use tempfile::tempdir;

pub fn apply_mask(
    input_path: &str,
    overlay_path: &str,
    output_path: &str,
    flip_overlay: bool,
    height_float: f32,
    fully_transparent: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_extension = Path::new(input_path).extension().and_then(|s| s.to_str()).unwrap_or("");
    
    let temp_dir_path = Path::new("temp_dir");
    
    if temp_dir_path.exists() {
        fs::remove_dir_all(temp_dir_path)?;
    }
    fs::create_dir(temp_dir_path)?;
    
    match input_extension.to_lowercase().as_str() {
        "png" | "jpg" | "jpeg" => apply_image_mask(input_path, overlay_path, output_path, flip_overlay, height_float, fully_transparent),
        "mp4" | "mov" => apply_video_mask(temp_dir_path, input_path, overlay_path, output_path, flip_overlay, height_float, fully_transparent),
        _ => Err("Unsupported file format".into()),
    }?;
    
    fs::remove_dir_all(temp_dir_path)?;
    
    Ok(())
}

fn apply_image_mask(
    input_path: &str,
    overlay_path: &str,
    output_path: &str,
    flip_overlay: bool,
    height_float: f32,
    fully_transparent: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let input_image = image::open(input_path)?;
    let mut overlay_image = image::open(overlay_path)?;
    let (input_width, input_height) = input_image.dimensions();
    let mask_height = (input_height as f32 * height_float) as u32;

    if flip_overlay {
        overlay_image = overlay_image.fliph();
    }

    let resized_overlay = resize_overlay(&overlay_image, input_width, mask_height);
    let mut output_image = ImageBuffer::new(input_width, input_height);

    for (x, y, pixel) in output_image.enumerate_pixels_mut() {
        let input_pixel = input_image.get_pixel(x, y);
        if y < mask_height && y < resized_overlay.height() && x < resized_overlay.width() {
            let overlay_pixel = resized_overlay.get_pixel(x, y);
            let mask_alpha = overlay_pixel[3];
            *pixel = if fully_transparent {
                apply_full_transparency(input_pixel, mask_alpha as f32 / 255.0)
            } else if mask_alpha == 0 {
                input_pixel
            } else {
                *overlay_pixel
            };
        } else {
            *pixel = input_pixel;
        }
    }

    output_image.save(output_path)?;
    Ok(())
}

fn resize_overlay(overlay: &DynamicImage, width: u32, height: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    overlay.resize_exact(width, height, image::imageops::FilterType::CatmullRom).to_rgba8()
}

fn apply_full_transparency(pixel: Rgba<u8>, mask_alpha: f32) -> Rgba<u8> {
    if mask_alpha > 0.0 {
        Rgba([pixel[0], pixel[1], pixel[2], 0])
    } else {
        pixel
    }
}

fn apply_video_mask(
    temp_dir: &Path,
    input_path: &str,
    overlay_path: &str,
    output_path: &str,
    flip_overlay: bool,
    height_float: f32,
    fully_transparent: bool,
) -> Result<(), Box<dyn std::error::Error>> {

    let temp_input_path = temp_dir.join("input.mp4");
    let temp_overlay_path = temp_dir.join("overlay.png");

    fs::copy(input_path, &temp_input_path)?;
    fs::copy(overlay_path, &temp_overlay_path)?;

    // Extract frames from input video
    let mut command = Command::new("ffmpeg");
    command.arg("-i").arg(&temp_input_path);
    command.arg("-vf");
    command.arg("fps=25");
    command.arg("-q:v");
    command.arg("2");
    command.arg(temp_dir.join("frame_%03d.png"));

    // println!("Executing command: {:?}", command);
    let output = command.output()?;
    // println!("Command output: {:?}", output);
    if!output.status.success() {
        return Err(format!("FFmpeg command failed: {:#?}", output).into());
    }

    // Apply mask to each frame in parallel
    let frame_paths: Vec<_> = fs::read_dir(temp_dir)?
       .filter_map(|entry| entry.ok())
       .filter(|entry| entry.file_type().ok().map_or(false, |ft| ft.is_file()))
       .map(|entry| entry.path())
       .filter(|path| path.file_name().unwrap().to_str().unwrap().starts_with("frame_"))
       .collect();

    println!("Applying mask to {} frames", frame_paths.len());
    frame_paths.par_iter().for_each(|path| {
        let output_path = temp_dir.join(format!("output_{:03}", path.file_name().unwrap().to_str().unwrap()));
        // println!("Applying mask to frame: {:?}", path);
        apply_image_mask(
            path.to_str().unwrap(),
            temp_overlay_path.to_str().unwrap(),
            output_path.to_str().unwrap(),
            flip_overlay,
            height_float,
            fully_transparent,
        )
       .unwrap();
        // println!("Mask applied to frame: {:?}", path);
    });

    // Combine edited frames into output video
    let mut command = Command::new("ffmpeg");
    command.arg("-framerate").arg("25");
    command.arg("-i").arg(temp_dir.join("output_frame_%03d.png"));
    command.arg("-c:v");
    command.arg("libvpx-vp9");
    command.arg("-pix_fmt");
    command.arg("yuva420p"); // Use yuva420p for alpha channels
    command.arg("-crf");
    command.arg("18");
    command.arg("-y"); // Add this option to force overwriting
    command.arg(output_path);

    // println!("Executing command: {:?}", command);
    let output = command.output()?;
    // println!("Command output: {:?}", output);
    if!output.status.success() {
        return Err(format!("FFmpeg command failed: {:#?}", output).into());
    }

    Ok(())
}

pub fn video_format_changer(input_filename: &String, output_filename: &String) -> Output {
    let output = Command::new("ffmpeg")
        .args(&[
            "-i", input_filename,
            "-c:v", "libx264",
            "-preset", "medium",
            "-crf", "23",
            "-c:a", "aac",
            "-b:a", "128k",
            "-fs", "100M",
            output_filename
        ])
        .output()
        .expect("Failed to execute FFmpeg command.");
    output
}

pub async fn video_convert(new_message: Message, ctx: serenity::prelude::Context, attachment: Attachment) {
    let mut msg = new_message.reply_ping(&ctx.http, format!("Converting {} to MP4!", attachment.filename)).await.unwrap();
    let input_filename = format!("./tmp/input_{}.tmp", Uuid::new_v4());
    let output_filename = format!("./tmp/output_{}.mp4", Uuid::new_v4());

    // Download the file
    let response = REQWEST_CLIENT.get(&attachment.url).send().await.unwrap();
    let bytes = response.bytes().await.unwrap();
    let mut file = std::fs::File::create(&input_filename).expect("Failed to create input file");
    file.write_all(&bytes).expect("Failed to write input file");

    // Convert the video using FFmpeg
    let output = video_format_changer(&input_filename, &output_filename);

    if output.status.success() {
        let file = serenity::all::CreateAttachment::path(&output_filename).await.unwrap();
        let build = EditMessage::new().new_attachment(file).content("Done!");
        match msg.edit(&ctx.http, build).await {
            Ok(()) => (),
            Err(_) => {msg.edit(&ctx.http, EditMessage::new().content("Message failed to edit, file may have been too large!")).await.unwrap(); ()} 
        };
    } else {
        println!("FFmpeg conversion failed: {:?}", String::from_utf8_lossy(&output.stderr));
        let _ = new_message.channel_id.say(&ctx.http, "Failed to convert the video.").await;
        let _ = std::fs::remove_file(&input_filename);
    }

    let _ = std::fs::remove_file(&input_filename);
    let _ = std::fs::remove_file(&output_filename);
}

pub fn image_to_png_converter(input_filename: &str, output_filename: &str) -> Output {
    let output = Command::new("ffmpeg")
        .args(&[
            "-i", input_filename,
            "-format", "png",
            "-lossless", "1",
            "-fs", "100M",
            output_filename
        ])
        .output()
        .expect("Failed to execute FFmpeg command.");
    
    output
}

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum QualityPreset {
    BestQuality,
    HighQuality,
    StandardQuality,
    LowQuality,
    LowestQuality,
    FastConversion,
    SmallFileSize,
    LargeFileSize,
    HighFPS,
    LowFPS,
    MaxColors,
    MinColors,
    NoDither,
    MaxDither,
    Retro,
    Vintage,
    Vibrant,
    Muted,
}

pub fn video_to_gif_converter(input_filename: &str, output_filename: &str, preset: QualityPreset) -> std::io::Result<()> {
    let (fps, colors, compression, quality, dither, bayer_scale, scale, additional_filters) = match preset {
        QualityPreset::BestQuality => ("30", "256", "6", "100", "sierra2_4a", "0", "1920:-1", ""),
        QualityPreset::HighQuality => ("24", "256", "7", "95", "floyd_steinberg", "3", "1280:-1", ""),
        QualityPreset::StandardQuality => ("20", "192", "8", "85", "floyd_steinberg", "3", "720:-1", ""),
        QualityPreset::LowQuality => ("15", "128", "9", "75", "bayer", "2", "480:-1", ""),
        QualityPreset::LowestQuality => ("10", "64", "9", "60", "bayer", "1", "320:-1", ""),
        QualityPreset::FastConversion => ("15", "128", "9", "75", "bayer", "2", "480:-1", ""),
        QualityPreset::SmallFileSize => ("10", "64", "9", "60", "bayer", "1", "320:-1", ""),
        QualityPreset::LargeFileSize => ("30", "256", "6", "100", "sierra2_4a", "0", "1920:-1", ""),
        QualityPreset::HighFPS => ("60", "192", "8", "90", "floyd_steinberg", "3", "1080:-1", ""),
        QualityPreset::LowFPS => ("10", "192", "8", "85", "floyd_steinberg", "3", "720:-1", ""),
        QualityPreset::MaxColors => ("24", "256", "7", "95", "none", "0", "1080:-1", ""),
        QualityPreset::MinColors => ("15", "32", "9", "75", "bayer", "2", "480:-1", ""),
        QualityPreset::NoDither => ("24", "256", "7", "90", "none", "0", "720:-1", ""),
        QualityPreset::MaxDither => ("24", "128", "8", "85", "sierra2_4a", "5", "720:-1", ""),
        QualityPreset::Retro => ("12", "16", "9", "80", "none", "0", "240:-1", "pixelate=24:24:0:0"),
        QualityPreset::Vintage => ("18", "64", "8", "85", "floyd_steinberg", "3", "640:-1", "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131,eq=saturation=0.7:gamma=1.2"),
        QualityPreset::Vibrant => ("24", "256", "7", "95", "floyd_steinberg", "3", "1080:-1", "eq=saturation=1.3:contrast=1.2"),
        QualityPreset::Muted => ("24", "192", "8", "90", "floyd_steinberg", "3", "720:-1", "eq=saturation=0.8:brightness=0.05"),
    };

    // Create a temporary directory for storing intermediate files
    let temp_dir = tempdir().expect("Failed to create temporary directory");
    let temp_path = temp_dir.path();

    // Split the video into 10-second segments
    let segment_duration = 10;
    let segment_pattern = temp_path.join("segment_%03d.mp4").to_str().unwrap().to_string();
    
    Command::new("ffmpeg")
        .args(&[
            "-i", input_filename,
            "-c", "copy",
            "-f", "segment",
            "-segment_time", &segment_duration.to_string(),
            "-reset_timestamps", "1",
            &segment_pattern,
        ])
        .output()
        .expect("Failed to split video into segments");

    // Process segments incrementally
    let mut processed_segments = Vec::new();
    for entry in fs::read_dir(temp_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("mp4") {
            let output_gif = path.with_extension("gif");
            let filter_complex = format!(
                "[0:v] fps={fps},scale={scale}:flags=lanczos{} [scaled];
                [scaled] split [a][b];
                [a] palettegen=max_colors={colors}:reserve_transparent=0:stats_mode=diff [p];
                [b][p] paletteuse=new=1:dither={dither}:bayer_scale={bayer_scale}:diff_mode=rectangle",
                if additional_filters.is_empty() { String::new() } else { format!(",{}", additional_filters) }
            );

            Command::new("ffmpeg")
                .args(&[
                    "-i", path.to_str().unwrap(),
                    "-filter_complex", &filter_complex,
                    "-compression_level", compression,
                    "-quality", quality,
                    output_gif.to_str().unwrap(),
                ])
                .output()
                .expect("Failed to convert segment to GIF");

            processed_segments.push(output_gif);

            // Combine processed segments when we have a certain number (e.g., 5)
            if processed_segments.len() >= 5 {
                combine_gifs(&processed_segments, temp_path, output_filename)?;
                processed_segments.clear();
            }
        }
    }

    // Combine any remaining segments
    if !processed_segments.is_empty() {
        combine_gifs(&processed_segments, temp_path, output_filename)?;
    }

    // Clean up temporary files
    temp_dir.close().expect("Failed to clean up temporary directory");

    Ok(())
}

fn combine_gifs(segments: &[PathBuf], temp_path: &Path, output_filename: &str) -> std::io::Result<()> {
    let concat_list = temp_path.join("concat_list.txt");
    let mut concat_file = fs::File::create(&concat_list)?;
    for gif in segments {
        writeln!(concat_file, "file '{}'", gif.to_str().unwrap())?;
    }

    let temp_output = temp_path.join("temp_output.gif");
    Command::new("ffmpeg")
        .args(&[
            "-f", "concat",
            "-safe", "0",
            "-i", concat_list.to_str().unwrap(),
            "-filter_complex", "split[s0][s1];[s0]palettegen[p];[s1][p]paletteuse",
            temp_output.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to concatenate GIF segments");

    // Append the temp_output to the final output file
    if Path::new(output_filename).exists() {
        let final_concat_list = temp_path.join("final_concat_list.txt");
        let mut final_concat_file = fs::File::create(&final_concat_list)?;
        writeln!(final_concat_file, "file '{}'", output_filename)?;
        writeln!(final_concat_file, "file '{}'", temp_output.to_str().unwrap())?;

        Command::new("ffmpeg")
            .args(&[
                "-f", "concat",
                "-safe", "0",
                "-i", final_concat_list.to_str().unwrap(),
                "-c", "copy",
                "-fs", "100M",
                &format!("{}.tmp", output_filename),
            ])
            .output()
            .expect("Failed to append to final GIF");

        fs::rename(format!("{}.tmp", output_filename), output_filename)?;
    } else {
        fs::rename(temp_output, output_filename)?;
    }

    Ok(())
}

pub fn png_to_gif_converter(input_filename: &str, output_filename: &str, preset: QualityPreset) -> std::io::Result<()> {
    let (colors, compression, quality, dither, bayer_scale, scale, additional_filters) = match preset {
        QualityPreset::BestQuality => ("256", "6", "100", "sierra2_4a", "0", "1920:-1", ""),
        QualityPreset::HighQuality => ("256", "7", "95", "floyd_steinberg", "3", "1280:-1", ""),
        QualityPreset::StandardQuality => ("192", "8", "85", "floyd_steinberg", "3", "720:-1", ""),
        QualityPreset::LowQuality => ("128", "9", "75", "bayer", "2", "480:-1", ""),
        QualityPreset::LowestQuality => ("64", "9", "60", "bayer", "1", "320:-1", ""),
        QualityPreset::FastConversion => ("128", "9", "75", "bayer", "2", "480:-1", ""),
        QualityPreset::SmallFileSize => ("64", "9", "60", "bayer", "1", "320:-1", ""),
        QualityPreset::LargeFileSize => ("256", "6", "100", "sierra2_4a", "0", "1920:-1", ""),
        QualityPreset::HighFPS => ("192", "8", "90", "floyd_steinberg", "3", "1080:-1", ""),
        QualityPreset::LowFPS => ("192", "8", "85", "floyd_steinberg", "3", "720:-1", ""),
        QualityPreset::MaxColors => ("256", "7", "95", "none", "0", "1080:-1", ""),
        QualityPreset::MinColors => ("32", "9", "75", "bayer", "2", "480:-1", ""),
        QualityPreset::NoDither => ("256", "7", "90", "none", "0", "720:-1", ""),
        QualityPreset::MaxDither => ("128", "8", "85", "sierra2_4a", "5", "720:-1", ""),
        QualityPreset::Retro => ("16", "9", "80", "none", "0", "240:-1", "pixelate=24:24:0:0"),
        QualityPreset::Vintage => ("64", "8", "85", "floyd_steinberg", "3", "640:-1", "colorchannelmixer=.393:.769:.189:0:.349:.686:.168:0:.272:.534:.131,eq=saturation=0.7:gamma=1.2"),
        QualityPreset::Vibrant => ("256", "7", "95", "floyd_steinberg", "3", "1080:-1", "eq=saturation=1.3:contrast=1.2"),
        QualityPreset::Muted => ("192", "8", "90", "floyd_steinberg", "3", "720:-1", "eq=saturation=0.8:brightness=0.05"),
    };

    let filter_complex = format!(
        "scale={scale}:flags=lanczos{},split[a][b];[a]palettegen=max_colors={colors}:reserve_transparent=1:stats_mode=full[p];[b][p]paletteuse=new=1:dither={dither}:bayer_scale={bayer_scale}:diff_mode=rectangle",
        if additional_filters.is_empty() { String::new() } else { format!(",{}", additional_filters) }
    );

    Command::new("ffmpeg")
        .args(&[
            "-i", input_filename,
            "-filter_complex", &filter_complex,
            "-loop", "0",
            "-compression_level", compression,
            "-quality", quality,
            "-fs", "100M",
            output_filename
        ])
        .output()?;
   
    Ok(())
}