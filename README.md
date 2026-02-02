# Gibberlink Decoder

Inspired by this youtube video where two AI agents switch to speaking in Gibberlink:

https://www.youtube.com/watch?v=EtNagNezo8w

I wanted to be able to translate the audio back into English to ensure that our AI friends really were saying what was on the screen.

To run, please refer to the commands below:

```
  # Basic usage                                                                                                                                                                                                   
  cargo run --release                                                                                                                                                                                             
                                                                                                                                                                                                                  
  # Verbose output                                                                                                                                                                                                
  cargo run --release -- --verbose                                                                                                                                                                                
                                                                                                                                                                                                                  
  # Save to file                                                                                                                                                                                                  
  cargo run --release -- --output decoded.txt                                                                                                                                                                     
                                                                                                                                                                                                                  
  # Custom input file                                                                                                                                                                                             
  cargo run --release -- --input path/to/audio.wav  
```
