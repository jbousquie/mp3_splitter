# MP3 Splitter

A Rust-based tool to split large MP3 files into smaller chunks of specified duration while maintaining proper MP3 encoding.

## Features

- Split MP3 files into chunks with a target duration (default: 10 minutes)
- Preserves proper MP3 encoding using the Symphonia audio library
- Maintains frame accuracy to ensure playable files in any MP3 player
- Copies and modifies ID3 tags from the source file to each chunk
- Updates title and track number metadata for each chunk

## Usage

```
cargo run -- <input_file> <chunk_minutes> <output_prefix>
```

### Example

```
cargo run -- podcast.mp3 10 podcast_part
```

This will split `podcast.mp3` into 10-minute chunks named:
- `podcast_part_001.mp3`
- `podcast_part_002.mp3`
- etc.

### Default Values

If no arguments are provided, the program will use these defaults:
- Input file: `podcastfi.mp3` (in the current directory)
- Chunk duration: 10 minutes
- Output prefix: `output_part`

## How It Works

The splitter uses a multi-pass approach:

1. **First Pass**: Read all audio packets and calculate timestamps and durations
2. **Second Pass**: Determine optimal chunk boundaries based on the target duration
3. **Third Pass**: Write packets to separate files and add appropriate metadata

## Dependencies

- **id3**: For handling MP3 metadata tags
- **symphonia**: For proper audio format detection and handling

## Output

All split files are saved in the `mp3_chunks` directory with sequential numbering. The directory is created if it doesn't exist.
Each file includes ID3 tags with:
- Updated title (original title + part number)
- Track number information
- Comment with split details
