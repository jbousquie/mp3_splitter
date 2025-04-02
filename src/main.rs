use std::fs;
use std::io;
use std::path::PathBuf;
use mp3_splitter::{split_mp3, SplitOptions, minutes_to_duration};

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
    
    let chunk_duration = minutes_to_duration(chunk_minutes);
    let folder_name = "mp3_chunks";
    match fs::create_dir(folder_name) {
        Ok(_) => println!("Directory {} created", folder_name),
        Err(_) => println!("Directory {} already exists", folder_name),
    }
    let output_dir = PathBuf::from(folder_name);
    
    // Create split options from parameters
    let options = SplitOptions {
        input_path: &input_file,
        chunk_duration,
        output_dir: &output_dir,
        prefix: &output_prefix,
    };
    
    // Execute the split operation
    match split_mp3(&options) {
        Ok(result) => {
            println!("MP3 file split completed successfully!");
            println!("Created {} chunks with total duration of {:.2} minutes", 
                     result.chunk_count,
                     result.total_duration.as_secs_f64() / 60.0);
        },
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}
