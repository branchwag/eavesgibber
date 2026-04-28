# Gibberlink Decoder

Inspired by [this YouTube video](https://www.youtube.com/watch?v=EtNagNezo8w) where two AI agents switch to speaking in Gibberlink.

I wanted to be able to translate the audio back into English to ensure that our AI friends really were saying what was on the screen.

## Usage

```sh
# Basic usage (decodes audio/gibberlink_trimmed.wav by default)
cargo run --release

# Decode a specific file
cargo run --release -- --input audio/gibberlink_trimmed.wav

# Save decoded text to a file
cargo run --release -- --input audio/gibberlink_trimmed.wav --output decoded.txt

# Verbose (see every detected symbol)
cargo run --release -- --input audio/gibberlink_trimmed.wav --verbose
```
