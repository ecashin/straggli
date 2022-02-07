use lv2_core::prelude::*;
use rand::Rng;
use urid::*;

const MAX_DELAY_IN_SECONDS: usize = 2;

struct Bounds {
    low: f64,
    high: f64,
}

impl Bounds {
    fn new(low: f64, high: f64) -> Self {
        Bounds { low, high }
    }

    fn mid_point(&self) -> f64 {
        (self.low + self.high) / 2.
    }

    fn contains(&self, x: f64) -> bool {
        if x < self.low {
            false
        } else if x > self.high {
            false
        } else {
            true
        }
    }

    fn span(&self) -> f64 {
        self.high - self.low
    }

    fn clamp(&self, x: f64) -> f64 {
        num::clamp(x, self.low, self.high)
    }

    fn rand_inside(&self) -> f64 {
        rand::thread_rng().gen_range(self.low..self.high)
    }
}

struct Walker {
    Position: Bounds,
    Acceleration: Bounds,
    Velocity: Bounds,
    AccelerationHold: i64,
}

#[derive(Copy, Clone)]
struct WalkerState {
    Acceleration: f64,
    Velocity: f64,
    Position: f64,
    AccelerationTTL: i64,
}

impl WalkerState {
    fn accelerating_left(&self) -> bool {
        self.Velocity < 0. && self.Acceleration < 0.
    }

    fn accelerating_right(&self) -> bool {
        self.Velocity > 0. && self.Acceleration > 0.
    }
}

impl Walker {
    fn new() -> Self {
        Walker {
            Position: Bounds::new(-100., -1.),
            Acceleration: Bounds::new(-0.4, 0.4),
            Velocity: Bounds::new(-5., 5.),
            AccelerationHold: 10,
        }
    }

    fn bounce(&self, current: WalkerState) -> (f64, f64) {
        let next_pos = current.Position + current.Velocity;
        let next_velo = if !self.Position.contains(next_pos) {
            self.Velocity.clamp(current.Velocity + current.Acceleration)
        } else {
            -1. * current.Velocity
        };
        (self.Position.clamp(next_pos), next_velo)
    }

    fn shy(&self, current: WalkerState) -> WalkerState {
        let next_acc = if current.accelerating_left() {
            current.Acceleration * (current.Position - self.Position.low) / self.Position.span()
        } else if current.accelerating_right() {
            current.Acceleration * (self.Position.high - current.Position) / self.Position.span()
        } else {
            current.Acceleration
        };
        WalkerState {
            Acceleration: next_acc,
            ..current
        }
    }

    fn step(&self, current: WalkerState) -> WalkerState {
        let (next_pos, next_velo) = self.bounce(current);
        let (next_acc, next_acc_ttl) = if current.AccelerationTTL > 0 {
            (current.Acceleration, current.AccelerationTTL - 1)
        } else {
            (self.Acceleration.rand_inside(), self.AccelerationHold)
        };
        self.shy(WalkerState {
            Acceleration: next_acc,
            Velocity: next_velo,
            Position: next_pos,
            AccelerationTTL: next_acc_ttl,
        })
    }
}

#[uri("https://github.com/RustAudio/rust-lv2/tree/master/docs/amp")]
struct JiglAgain {
    walkers: [Walker; 2],
    buffers: [Vec<f32>; 2],
    buf_pos: usize,
}

#[derive(PortCollection)]
struct JiglAgainPorts {
    gain: InputPort<Control>,
    input_left: InputPort<InPlaceAudio>,
    input_right: InputPort<InPlaceAudio>,
    output_left: OutputPort<InPlaceAudio>,
    output_right: OutputPort<InPlaceAudio>,
}

impl Plugin for JiglAgain {
    type Ports = JiglAgainPorts;
    type InitFeatures = ();
    type AudioFeatures = ();
    fn new(plugin_info: &PluginInfo, _features: &mut Self::InitFeatures) -> Option<Self> {
        let sr = plugin_info.sample_rate() as usize;
        Some(Self {
            buffers: [
                vec![0.; sr * MAX_DELAY_IN_SECONDS],
                vec![0.; sr * MAX_DELAY_IN_SECONDS],
            ],
            walkers: [Walker::new(), Walker::new()],
            buf_pos: 0,
        })
    }
    // What implementation details elided ?

    fn run(&mut self, ports: &mut JiglAgainPorts, _: &mut (), _: u32) {
        let coef = if *(ports.gain) > -90.0 {
            10.0_f32.powf(*(ports.gain) * 0.05)
        } else {
            0.0
        };

        let input = Iterator::zip(ports.input_left.iter(), ports.input_right.iter());
        let output = Iterator::zip(ports.output_left.iter(), ports.output_right.iter());
        for ((in_left, in_right), (out_left, out_right)) in Iterator::zip(input, output) {
            let a = in_left.get();
            let b = in_right.get();
            self.buffers[0][self.buf_pos] = a;
            self.buffers[1][self.buf_pos] = b;
            out_left.set(a * coef);
            out_right.set(b * coef);
            self.buf_pos += 1;
            if self.buf_pos > self.buffers[0].len() {
                self.buf_pos = 0;
            }
        }
    }
}

lv2_descriptors!(JiglAgain);
