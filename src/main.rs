#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use anyhow::{Context, Result};
use clap::Parser;
use hound;
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Decode ggwave audio from Gibberlink demo")]
struct Args {
    /// Path to the WAV file
    #[arg(short, long, default_value = "audio/gibberlink_trimmed.wav")]
    input: PathBuf,

    /// Output decoded messages to file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Show detailed debug information
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("🎵 Eavesgibber - Gibberlink Decoder");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("   Using native ggwave library\n");

    // Load audio
    println!("📂 Loading audio from {:?}", args.input);
    let (samples, sample_rate) = load_audio(&args.input)?;
    println!(
        "   {} samples @ {} Hz ({:.2} seconds)\n",
        samples.len(),
        sample_rate,
        samples.len() as f32 / sample_rate as f32
    );

    // Decode using ggwave
    println!("🔍 Decoding with ggwave library...\n");
    let messages = decode_with_ggwave(&samples, sample_rate, args.verbose)?;

    // Display results
    if messages.is_empty() {
        println!("❌ No messages decoded.\n");
        println!("Troubleshooting:");
        println!("  • Make sure the audio contains ggwave-encoded data");
        println!("  • Try the online decoder: https://ggwave-js.ggerganov.com/");
    } else {
        println!("✅ Found {} message(s):\n", messages.len());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        for (i, msg) in messages.iter().enumerate() {
            println!("\n📨 Message {} @ {:.2}s", i + 1, msg.timestamp);
            println!("   {}", msg.text);
        }

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

        if let Some(output) = &args.output {
            save_messages(&messages, output)?;
            println!("💾 Saved to {:?}\n", output);
        }
    }

    Ok(())
}

#[derive(Debug, Clone)]
struct DecodedMessage {
    timestamp: f32,
    text: String,
}

fn load_audio(path: &PathBuf) -> Result<(Vec<f32>, u32)> {
    let mut reader = hound::WavReader::open(path).context("Failed to open WAV file")?;
    let spec = reader.spec();
    let sample_rate = spec.sample_rate;

    let samples: Vec<f32> = match spec.channels {
        1 => reader
            .samples::<i16>()
            .map(|sample| {
                sample
                    .map(|s| s as f32 / 32768.0)
                    .context("Failed to read WAV sample")
            })
            .collect::<Result<Vec<_>>>()?,
        2 => {
            let mut mono = Vec::new();
            let mut iter = reader.samples::<i16>();
            while let Some(left) = iter.next() {
                let l = left.context("Failed to read left WAV channel sample")?;
                let r = iter
                    .next()
                    .context("Stereo WAV ended with an incomplete sample frame")?
                    .context("Failed to read right WAV channel sample")?;
                mono.push((l as f32 + r as f32) / 2.0 / 32768.0);
            }
            mono
        }
        _ => anyhow::bail!("Only mono or stereo audio is supported"),
    };

    Ok((samples, sample_rate))
}

fn decode_with_ggwave(
    samples: &[f32],
    sample_rate: u32,
    verbose: bool,
) -> Result<Vec<DecodedMessage>> {
    let mut messages = Vec::new();

    // Convert to i16 samples (ggwave often works better with integer samples)
    let samples_i16: Vec<i16> = samples
        .iter()
        .map(|&s| (s * 32767.0).clamp(-32768.0, 32767.0) as i16)
        .collect();

    unsafe {
        // Get default parameters and configure for our audio
        let mut params = ggwave_getDefaultParameters();
        params.sampleRateInp = sample_rate as f32;
        params.sampleRateOut = sample_rate as f32;
        params.sampleRate = sample_rate as f32;
        params.sampleFormatInp = ggwave_SampleFormat_GGWAVE_SAMPLE_FORMAT_I16;
        params.operatingMode = GGWAVE_OPERATING_MODE_RX as i32;
        params.samplesPerFrame = 1024;
        // Lower threshold for better detection of compressed audio
        params.soundMarkerThreshold = 2.0;

        // Create ggwave instance
        let instance = ggwave_init(params);
        if instance < 0 {
            anyhow::bail!("Failed to initialize ggwave (error code: {})", instance);
        }

        if verbose {
            println!("   ggwave instance created (id: {})", instance);
            println!("   Sample rate: {} Hz", sample_rate);
            println!("   Sample format: F32");
        }

        // Buffer for decoded data
        let mut decode_buffer = vec![0u8; 256];

        // Use frame-aligned chunk size (must be multiple of samplesPerFrame)
        let samples_per_frame = 1024;
        let chunk_size = samples_per_frame * 4; // Process multiple frames at a time

        let mut sample_offset = 0;
        let mut chunk_count = 0;

        while sample_offset + chunk_size <= samples_i16.len() {
            let chunk = &samples_i16[sample_offset..sample_offset + chunk_size];

            // Feed samples to ggwave using ndecode for explicit buffer size
            // Note: waveformSize is in BYTES, not samples, for i16 that's samples * 2
            let result = ggwave_ndecode(
                instance,
                chunk.as_ptr() as *const std::ffi::c_void,
                (chunk.len() * 2) as i32, // i16 = 2 bytes per sample
                decode_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                decode_buffer.len() as i32,
            );

            if result > 0 {
                // Successful decode!
                let decoded_len = result as usize;
                let text = String::from_utf8_lossy(&decode_buffer[..decoded_len]).to_string();

                let timestamp = sample_offset as f32 / sample_rate as f32;

                if verbose {
                    println!(
                        "   ✅ Decoded at {:.2}s: {} bytes - \"{}\"",
                        timestamp,
                        decoded_len,
                        text.trim()
                    );
                }

                // Strip protocol prefix (e.g., "V5$" or "6P$")
                let clean_text = strip_protocol_prefix(text.trim());
                messages.push(DecodedMessage {
                    timestamp,
                    text: clean_text,
                });

                // Clear buffer for next message
                decode_buffer.fill(0);
            }

            sample_offset += chunk_size;
            chunk_count += 1;
        }

        // Process remaining samples (might have partial data)
        if sample_offset < samples_i16.len() {
            // Pad to make it a multiple of frame size
            let remaining = &samples_i16[sample_offset..];
            let mut padded = remaining.to_vec();
            let pad_needed = samples_per_frame - (remaining.len() % samples_per_frame);
            if pad_needed < samples_per_frame {
                padded.extend(std::iter::repeat(0i16).take(pad_needed));
            }

            let result = ggwave_ndecode(
                instance,
                padded.as_ptr() as *const std::ffi::c_void,
                (padded.len() * 2) as i32, // i16 = 2 bytes per sample
                decode_buffer.as_mut_ptr() as *mut std::ffi::c_void,
                decode_buffer.len() as i32,
            );

            if result > 0 {
                let decoded_len = result as usize;
                let text = String::from_utf8_lossy(&decode_buffer[..decoded_len]).to_string();
                let timestamp = sample_offset as f32 / sample_rate as f32;

                if verbose {
                    println!(
                        "   ✅ Decoded at {:.2}s: {} bytes - \"{}\"",
                        timestamp,
                        decoded_len,
                        text.trim()
                    );
                }

                // Strip protocol prefix (e.g., "V5$" or "6P$")
                let clean_text = strip_protocol_prefix(text.trim());
                messages.push(DecodedMessage {
                    timestamp,
                    text: clean_text,
                });
            }
        }

        // Clean up
        ggwave_free(instance);

        if verbose {
            println!(
                "\n   Processed {} chunks ({} samples total)",
                chunk_count,
                samples.len()
            );
        }
    }

    Ok(messages)
}

fn strip_protocol_prefix(text: &str) -> String {
    // ggwave prepends a 3-character protocol identifier like "V5$" or "6P$"
    if text.len() > 3 && text.chars().nth(2) == Some('$') {
        text[3..].to_string()
    } else {
        text.to_string()
    }
}

fn save_messages(messages: &[DecodedMessage], path: &PathBuf) -> Result<()> {
    use std::fs::File;
    use std::io::Write;

    let mut file = File::create(path)?;
    writeln!(file, "Gibberlink Decoded Messages")?;
    writeln!(file, "===========================\n")?;

    for (i, msg) in messages.iter().enumerate() {
        writeln!(file, "Message {}: @ {:.2}s", i + 1, msg.timestamp)?;
        writeln!(file, "{}\n", msg.text)?;
    }

    Ok(())
}
