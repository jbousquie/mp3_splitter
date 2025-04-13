# MP3 Splitter

A Rust-based tool and library to split large MP3 files into smaller chunks of specified duration while maintaining proper MP3 encoding. This code is mainly produced by Copilot + Claude Sonnet 3.7, then reviewed, fixed and tested by me.

## Features

- Split MP3 files into chunks with a target duration (default: 10 minutes)
- Preserves proper MP3 encoding using the Symphonia audio library
- Maintains frame accuracy to ensure playable files in any MP3 player
- Copies and modifies ID3 tags from the source file to each chunk
- Updates title and track number metadata for each chunk
- Available as both a command-line tool and a library for integration into other Rust projects

## Command-line Usage

```
cargo run <input_file> <chunk_minutes> <output_prefix>
```

### Example

```
cargo run podcast.mp3 10 podcast_part
```

This will split `podcast.mp3` into 10-minute chunks named:
- `podcast_part_001.mp3`
- `podcast_part_002.mp3`
- etc.

### Default Values

If no arguments are provided, the program will use these defaults:
- Input file: `audiofile.mp3` (in the current directory)
- Chunk duration: 10 minutes
- Output prefix: `audiofile_part`

## Library Usage

You can use MP3 Splitter as a library in your Rust projects.

### Add as a dependency

Add this to your `Cargo.toml`:

```toml
[dependencies]
mp3_splitter = { path = "/path/to/mp3_splitter" }
# Or if published to crates.io:
# mp3_splitter = "0.1.0"
```

### Example usage

```rust
use mp3_splitter::{split_mp3, SplitOptions, minutes_to_duration};
use std::path::{Path, PathBuf};
use std::io;

fn main() -> io::Result<()> {
    // Set up the splitting configuration
    let options = SplitOptions {
        input_path: Path::new("podcast.mp3"),
        chunk_duration: minutes_to_duration(10), // 10 minute chunks
        output_dir: Path::new("audio_chunks"),
        prefix: "podcast_part",
    };
    
    // Perform the split
    match split_mp3(&options) {
        Ok(result) => {
            println!("Split into {} chunks", result.chunk_count);
            
            // Access information about the split
            println!("Total duration: {:.2} minutes", result.total_duration.as_secs_f64() / 60.0);
            
            // You can also access all output file paths
            for path in result.output_files {
                println!("Created: {}", path.display());
            }
        },
        Err(e) => eprintln!("Error: {}", e),
    }
    
    Ok(())
}
```

## How It Works

The splitter uses a multi-pass approach:

1. **First Pass**: Read all audio packets and calculate timestamps and durations
2. **Second Pass**: Determine optimal chunk boundaries based on the target duration
3. **Third Pass**: Write packets to separate files and add appropriate metadata

## Dependencies

- **id3**: For handling MP3 metadata tags
- **symphonia**: For proper audio format detection and handling

## Output

All split files are saved in the `audio_chunks` directory with sequential numbering. The directory is created if it doesn't exist.
Each file includes ID3 tags with:
- Updated title (original title + part number)
- Track number information
- Comment with split details

## API Documentation

### Main Types

- `SplitOptions`: Configuration options for the splitting process
- `SplitResult`: Contains information about the completed split
- `ChunkInfo`: Details about an individual chunk

### Main Functions

- `split_mp3(&options)`: Main function to split an MP3 file according to provided options
- `minutes_to_duration(minutes)`: Helper function to convert minutes to Duration

## Conversion

You can use for instance `ffmpeg` to convert a `wav` audio file into a `mp3` audio file if needed.

``` 
ffmpeg -i audio.wav -acodec mp3 audio.mp3
```
