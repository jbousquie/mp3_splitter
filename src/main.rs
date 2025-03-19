use std::fs::{self, File};
use std::io::{self, Write, BufWriter};
use std::path::{Path, PathBuf};
use id3::{Tag, TagLike, Version};
use symphonia::core::io::{MediaSourceStream, ReadOnlySource};
use symphonia::core::formats::FormatOptions;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use std::time::Duration;

/// Information about an audio chunk
struct ChunkInfo {
    start_time: Duration,
    end_time: Duration,
    packets: Vec<usize>, // Indices of packets in the global packets list
}

/// Split an MP3 file into chunks of specified duration
fn split_mp3(input_path: &Path, chunk_duration: Duration, output_dir: &Path, prefix: &str) -> io::Result<()> {
    println!("Processing file: {}", input_path.display());
    println!("Target chunk duration: {} seconds ({} minutes)", 
        chunk_duration.as_secs(), 
        chunk_duration.as_secs() / 60);
    
    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)?;
    }
    
    // Open the media source
    let file = Box::new(ReadOnlySource::new(File::open(input_path)?));
    let mss = MediaSourceStream::new(file, Default::default());
    
    // Create a hint to help with format detection
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    
    // Use default options
    let format_opts = FormatOptions::default();
    let metadata_opts = MetadataOptions::default();
    
    // Probe the format
    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &format_opts, &metadata_opts)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Error probing format: {}", e)))?;
    
    let mut format = probed.format;
    
    // Get the default track
    let track = format.default_track()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No default track found"))?;
    
    // Get codec parameters and time base
    let codec_params = track.codec_params.clone();
    let time_base = codec_params.time_base
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "No time base found"))?;
    
    // Read original ID3 tags
    let original_tag = Tag::read_from_path(input_path).ok();
    
    // Store all packets and their durations
    let mut packets = Vec::new();
    let mut packet_times = Vec::new();
    let mut total_duration = Duration::from_secs(0);
    
    // First pass: read all packets and calculate timestamps
    println!("First pass: reading packets and calculating timestamps...");
    while let Ok(packet) = format.next_packet() {
        // Calculate duration of this packet
        let frame_len = packet.dur;
        let packet_duration = Duration::from_secs_f64(
            frame_len as f64 * time_base.numer as f64 / time_base.denom as f64
        );
        
        total_duration += packet_duration;
        packet_times.push(total_duration);
        packets.push(packet);
    }
    
    if packets.is_empty() {
        return Err(io::Error::new(io::ErrorKind::Other, "No audio packets found"));
    }
    
    println!("Found {} packets, total duration: {:.2} seconds ({:.2} minutes)", 
        packets.len(), 
        total_duration.as_secs_f64(),
        total_duration.as_secs_f64() / 60.0
    );
    
    // Second pass: determine chunk boundaries
    println!("Second pass: determining chunk boundaries...");
    let mut chunks = Vec::new();
    let mut chunk_start_packet = 0;
    let mut chunk_start_time = Duration::from_secs(0);
    
    while chunk_start_packet < packets.len() {
        // Find the packet that would end this chunk
        let target_end_time = chunk_start_time + chunk_duration;
        
        // Find the packet index that's closest to our target end time
        let mut chunk_end_packet = chunk_start_packet;
        while chunk_end_packet < packets.len() && 
              (chunk_end_packet == chunk_start_packet || 
               packet_times[chunk_end_packet - 1] < target_end_time) {
            chunk_end_packet += 1;
        }
        
        // Ensure we include at least one packet
        if chunk_end_packet == chunk_start_packet {
            chunk_end_packet = chunk_start_packet + 1;
        }
        
        // Get the actual end time for this chunk
        let chunk_end_time = if chunk_end_packet < packets.len() {
            packet_times[chunk_end_packet - 1]
        } else {
            total_duration
        };
        
        // Create packet index list for this chunk
        let mut chunk_packets = Vec::new();
        for i in chunk_start_packet..chunk_end_packet {
            chunk_packets.push(i);
        }
        
        chunks.push(ChunkInfo {
            start_time: chunk_start_time,
            end_time: chunk_end_time,
            packets: chunk_packets,
        });
        
        // Move to next chunk
        chunk_start_packet = chunk_end_packet;
        chunk_start_time = chunk_end_time;
        
        // Break if we've processed all packets
        if chunk_start_packet >= packets.len() {
            break;
        }
    }
    
    println!("Splitting into {} chunks:", chunks.len());
    
    // Debug output to check chunk durations
    for (i, chunk) in chunks.iter().enumerate() {
        let duration = (chunk.end_time - chunk.start_time).as_secs_f64();
        println!("Chunk {} duration: {:.2} minutes ({:.2} seconds), packets: {}", 
            i+1, duration/60.0, duration, chunk.packets.len());
    }
    
    // Third pass: write chunks to files
    for (chunk_idx, chunk) in chunks.iter().enumerate() {
        let output_filename = format!("{}_{:03}.mp3", prefix, chunk_idx + 1);
        let output_path = output_dir.join(&output_filename);
        
        println!(
            "Writing chunk {}/{}: {} (duration: {:.2} minutes, {} packets)",
            chunk_idx + 1,
            chunks.len(),
            output_filename,
            (chunk.end_time - chunk.start_time).as_secs_f64() / 60.0,
            chunk.packets.len()
        );
        
        let mut output = BufWriter::new(File::create(&output_path)?);
        
        // Write all packets for this chunk
        for &packet_idx in &chunk.packets {
            output.write_all(&packets[packet_idx].data)?;
        }
        output.flush()?;
        
        // Apply ID3 tags with modifications
        if let Some(ref tag) = original_tag {
            let mut new_tag = tag.clone();
            
            // Update the title to include part number
            if let Some(title) = new_tag.title() {
                let new_title = format!("{} (Part {}/{})", title, chunk_idx + 1, chunks.len());
                new_tag.set_title(new_title);
            }
            
            // Set track number
            new_tag.set_track((chunk_idx + 1) as u32);
            

            
            // Write the tag to the new file
            if let Err(e) = new_tag.write_to_path(&output_path, Version::Id3v24) {
                eprintln!("Warning: Failed to write ID3 tags: {}", e);
            }
        }
    }
    
    println!("Successfully split MP3 file into {} chunks in directory: {}", 
        chunks.len(), output_dir.display());
    
    Ok(())
}

fn main() -> io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    
    let (input_file, chunk_minutes, output_prefix) = if args.len() >= 4 {
        let file = PathBuf::from(&args[1]);
        let minutes = args[2].parse::<u64>().unwrap_or(10);
        let prefix = &args[3];
        (file, minutes, prefix.clone())
    } else {
        // Default values
        let default_input = PathBuf::from("audiofile.mp3");
        println!("Using default parameters:");
        println!("  Input file: {}", default_input.display());
        println!("  Chunk duration: 10 minutes");
        println!("  Output prefix: audiofile_part");
        println!("  Output folder: mp3_chunks");
        println!();
        println!("To specify custom parameters, use: cargo run -- <input_file> <chunk_minutes> <output_prefix>");
        
        (default_input, 10, "audiofile_part".to_string())
    };
    
    let chunk_duration = Duration::from_secs(chunk_minutes * 60);
    let folder_name = "mp3_chunks";
    match fs::create_dir(folder_name) {
        Ok(_) => println!("Directory {} created", folder_name),
        Err(_) => println!("Directory {} already exists", folder_name),
    }
    let output_dir = PathBuf::from(folder_name);
    
    match split_mp3(&input_file, chunk_duration, &output_dir, &output_prefix) {
        Ok(_) => println!("MP3 file split completed successfully!"),
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}
