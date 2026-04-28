# Gibberlink Decoder

Inspired by [this YouTube video](https://www.youtube.com/watch?v=EtNagNezo8w) where two AI agents switch to speaking in Gibberlink.

I wanted to be able to translate the audio back into English to ensure that our AI friends really were saying what was on the screen.

## Usage

```sh
# Basic usage
cargo run --release

# Decode a specific file
cargo run --release -- --input audio/gibberlink_trimmed.wav

# With protocol analysis
cargo run --release -- --input audio/gibberlink_trimmed.wav --analyze

# Save binary output
cargo run --release -- -i audio/gibberlink_trimmed.wav -o decoded.txt --save-binary

# Verbose (see every detected symbol)
cargo run --release -- -i audio/gibberlink_trimmed.wav --verbose

# Save decoded text to a file
cargo run --release -- --output decoded.txt
```
