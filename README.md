# rusty-wav-effects
sound postprocessing in rust

Just Ben fooling around with .wav file postprocessing as a way to learn about Rust.

current limitations:
- input and output must be .wav files at present
- utilizes https://github.com/ruuda/hound repo to do .wav file I/O (thank you Ruud!)
- all input parameters must be specified in CLI at present (still learning clap crate)
- only 16-bit-integer single-channel .wav format is supported.

Since I am not doing any live sound yet, I haven't really optimized code.

Example:

```
# cargo build
# ln -sv target/debug/sound_xform_hound sound_xform
# ./sound_xform --help
# ./sound_xform -i t2.wav -o ss4.wav -t delay -a 0.3 -w 1.0
### play the resulting .wav file using pipewire (can also use pacat for pulse audio)
# pw-cat -p ss4.wav
input wav file: t2.wav
output wav file: ss4.wav
transform: delay
amplitude: 0.3
wavelength (sec): 1
```

short-form Input parameters:
- -i -- input .wav file pathname
- -o -- output .wav file pathname
- -t -- transform type (tremolo, delay or none at present)
- -a -- amplitude (0.0 to 1.0 where 1.0 is all effect, 0.0 is no effect)
- -w -- wavelength = time parameter (more later)

# sound transform types

We will go through each transform type in subsections below, but first let's discuss the amplitude parameter,
which is common to all.   We can't insert an effect without making room for it - in other words, we have to keep the sample level from accidentally hitting its upper or lower bound, as this will cause distortion (maybe another effect later?)
To do this, we attenuate the original sample by multiplying it by (1.0 - a) where *a* is the amplitude parameter.  
We then compute the effect contribution to the sample by multiplying it by *a*.  We then add these two terms and are
guaranteed that the resulting sample will not reach the maximum/minimum sample value unless the original sample did.

Now for transform types:

## tremolo
This effect modulates sound with sine wave to simulate an old tremolo effects box.  Actually it uses absolute value of sin, 
because without absolute value the amplitude of tremolo and original sample can cancel for sin inputs between PI and 2*PI.
The wavelength is really the 1/2 wavelength for this reason.

## delay

This effect applies a digital delay similar to a delay box.  The wavelength parameter here is just the delay time.  Note that since the delay is computed in the forward time direction, a sample can effect more than 1 sample in the future, so be careful with that amplitude ;-)

## none

This effect just adjusts the volume downwards or upwards (if upwards, be careful of clipping and damaging your equipment, including your ears!).

