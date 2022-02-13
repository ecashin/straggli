# Straggli - Randomized Delay Lines

This LV2 plugin is based on the "walkers" experiments
in the `LoopMyWav` repository.
Its name evokes an image of people who are straggling,
getting left behind by a group and doing random things or wandering rogue.

## Overview

This is a noisy plugin
that helps sound designers find varied and novel kinds of audio degredation.

For each channel of a stereo signal,
a "walker" has a random acceleration with a timeout.
The "position" of the walker straggles behind the advancing
incoming-audio position.
The walker's acceleration modifies the velocity (forward or backward)
of its relative position during its "time to live".
Then a new random acceleration is generated.

## Controls

The plugin presents controls that allow the user to place
bounds on the walker's position, expressed as a millisecond delay
with respect to the incoming audio.
Also the user can bound other parameters in units of samples
per sample.
E.g., velocity is the change in relative position in samples
per every incoming stereo sample.

A wet-mix control allows the unmodified signal to be included
when the setting is less than 100.

## Building

To build, use Rust's `cargo`.

    cargo build

Then copy the resulting `.so` file from the target area to your
bundle directory, where the turtle files in `straggli.lv2` should
also appear.
