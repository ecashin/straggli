# Notes for Whooshi

The LoopMyWav walkers implement the guts
of whooshi.

## Key Terms and Concepts

The F# implementation uses closures and updated immutable structs.

Rust has struct updating.
Probably more conventional for rust would be the use
of methods instead of closures.

"State" is composed of acceleration, "speed" (velocity),
"pos" (position of the read head for the audio delay),
and "delay" (number of steps before update--see "Name Refinement" below).

A walker produces a new State from an old one at every "step".
The new State might be the same as the old one.
A walker is defined by a WalkerDef,
composed of MostDelay (greatest allowed number of steps before state change),
LeastDelay,
AccA (least acceleration),
AccB (greatest acceleration),
SpeedA (most negative velocity),
SpeedB (most positive velocity),
UpdateDelay (initial number of steps before acceleration change).

State initially has a delay equal to UpdateDelay
but has a dynamic value for that, resetting if it becomes negative.

A moving delay line will "shy" away from the limits of its range
by decreasing its absolute acceleration
in proportion to limit proximity.

A moving delay line will "bounce" off the limits, changing direction.

## Name Refinement

Some names inside need refinement.

"Delay" is the number of steps before an acceleration change.
A better name wouldn't confuse a reader who associates
that term with an audio echo effect.
Alternatives:

* ConstAccSteps
* AccHoldSteps - number of acceleration hold steps

"Speed" has one-dimensional direction,
so "velocity" is a better name.

* Velocity

"UpdateDelay" 

## Miscellany

There's a soft clipping function in Grain.fs.

Hyper.fs contains hyperparameter searching code.

