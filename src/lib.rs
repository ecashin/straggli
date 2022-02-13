use lv2_core::prelude::*;
use rand::Rng;
use urid::*;

const MAX_DELAY_IN_SECONDS: usize = 2;

#[derive(Debug)]
struct Bounds {
    low: f32,
    high: f32,
}

impl Bounds {
    fn new(low: f32, high: f32) -> Self {
        Bounds { low, high }
    }

    fn mid_point(&self) -> f32 {
        (self.low + self.high) / 2.
    }

    fn contains(&self, x: f32) -> bool {
        x >= self.low && x <= self.high
    }

    fn span(&self) -> f32 {
        self.high - self.low
    }

    fn clamp(&self, x: f32) -> f32 {
        num::clamp(x, self.low, self.high)
    }

    fn rand_inside(&self) -> f32 {
        rand::thread_rng().gen_range(self.low..self.high)
    }
}

#[derive(Debug)]
struct Walker {
    position: Bounds,
    acceleration: Bounds,
    velocity: Bounds,
    acceleration_hold: i32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
struct WalkerState {
    acceleration: f32,
    velocity: f32,
    position: f32,
    acceleration_ttl: i32,
}

impl WalkerState {
    fn new_from_walker(walker: &Walker) -> WalkerState {
        WalkerState {
            acceleration: walker.acceleration.rand_inside(),
            velocity: walker.velocity.rand_inside(),
            position: walker.position.rand_inside(),
            acceleration_ttl: walker.acceleration_hold,
        }
    }

    fn accelerating_left(&self) -> bool {
        self.velocity < 0. && self.acceleration < 0.
    }

    fn accelerating_right(&self) -> bool {
        self.velocity > 0. && self.acceleration > 0.
    }
}

impl Walker {
    fn new() -> Self {
        println!("in Walker::new");
        Walker {
            position: Bounds::new(-8000., -1.),
            acceleration: Bounds::new(-1., 1.),
            velocity: Bounds::new(-50., 50.),
            acceleration_hold: 10,
        }
    }

    fn bounce(&self, current: WalkerState) -> (f32, f32) {
        let next_pos = current.position + current.velocity;
        let next_velo = if self.position.contains(next_pos) {
            self.velocity.clamp(current.velocity + current.acceleration)
        } else {
            -1. * current.velocity
        };
        (self.position.clamp(next_pos), next_velo)
    }

    fn shy(&self, current: WalkerState) -> WalkerState {
        let next_acc = if current.accelerating_left()
            && current.position < self.position.mid_point()
        {
            current.acceleration * (current.position - self.position.low) / self.position.span()
        } else if current.accelerating_right() && current.position > self.position.mid_point() {
            current.acceleration * (self.position.high - current.position) / self.position.span()
        } else {
            current.acceleration
        };
        WalkerState {
            acceleration: next_acc,
            ..current
        }
    }

    fn step(&self, current: WalkerState) -> WalkerState {
        let (next_pos, next_velo) = self.bounce(current);
        let (next_acc, next_acc_ttl) = if current.acceleration_ttl > 0 {
            (current.acceleration, current.acceleration_ttl - 1)
        } else {
            (self.acceleration.rand_inside(), self.acceleration_hold)
        };
        self.shy(WalkerState {
            acceleration: next_acc,
            velocity: next_velo,
            position: next_pos,
            acceleration_ttl: next_acc_ttl,
        })
    }

    fn get(&self, s: &WalkerState, buf: &[f32], pos: usize) -> f32 {
        // this position can be negative
        let position = pos as i64 + s.position as i64;
        // adding buffer length doesn't change value modulo buffer length
        let positive_position = position + buf.len() as i64;
        buf[positive_position as usize % buf.len()]
    }
}

#[uri("https://github.com/ecashin/jiglagain")]
struct JiglAgain {
    sample_rate: usize,
    walkers: [Walker; 2],
    walker_states: [WalkerState; 2],
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
    max_delay_ms: InputPort<Control>,
    max_abs_acc: InputPort<Control>,
    max_abs_velo: InputPort<Control>,
    wet_mix: InputPort<Control>,
}

impl JiglAgain {
    fn pos_from_delay_ms(&self, max_delay_ms: f32) -> f32 {
        let delay_seconds = max_delay_ms / 1000.;
        -delay_seconds * (self.sample_rate as f32)
    }
}

impl Plugin for JiglAgain {
    type Ports = JiglAgainPorts;
    type InitFeatures = ();
    type AudioFeatures = ();
    fn new(plugin_info: &PluginInfo, _features: &mut Self::InitFeatures) -> Option<Self> {
        println!("in JiglAgain Plugin new");
        let sr = plugin_info.sample_rate() as usize;
        let walkers = [Walker::new(), Walker::new()];
        let walker_states = [
            WalkerState::new_from_walker(&walkers[0]),
            WalkerState::new_from_walker(&walkers[1]),
        ];
        Some(Self {
            sample_rate: sr,
            buffers: [
                vec![0.; sr * MAX_DELAY_IN_SECONDS],
                vec![0.; sr * MAX_DELAY_IN_SECONDS],
            ],
            walkers,
            walker_states,
            buf_pos: 0,
        })
    }

    fn run(&mut self, ports: &mut JiglAgainPorts, _: &mut (), _: u32) {
        let input = Iterator::zip(ports.input_left.iter(), ports.input_right.iter());
        let output = Iterator::zip(ports.output_left.iter(), ports.output_right.iter());
        let gain = if *(ports.gain) > -90.0 {
            10.0_f32.powf(*(ports.gain) * 0.05)
        } else {
            0.0
        };
        let wet_mix = *(ports.wet_mix) / 100.;
        let pos_low = self.pos_from_delay_ms(*(ports.max_delay_ms));
        let pos_high = self.walkers[0].position.high;
        if pos_low < pos_high {
            self.walkers[0].position.low = pos_low;
            self.walkers[1].position.high = pos_high;
        }

        let acc = *(ports.max_abs_acc);
        self.walkers[0].acceleration.low = -acc;
        self.walkers[1].acceleration.low = -acc;
        self.walkers[0].acceleration.high = acc;
        self.walkers[1].acceleration.high = acc;
        let velo = *(ports.max_abs_velo);
        self.walkers[0].velocity.low = -velo;
        self.walkers[1].velocity.low = -velo;
        self.walkers[0].velocity.high = velo;
        self.walkers[1].velocity.high = velo;

        for ((in_left, in_right), (out_left, out_right)) in Iterator::zip(input, output) {
            let a = in_left.get();
            let b = in_right.get();
            let aa = self.walkers[0].get(&self.walker_states[0], &self.buffers[0], self.buf_pos);
            let bb = self.walkers[1].get(&self.walker_states[1], &self.buffers[1], self.buf_pos);
            self.walker_states[0] = self.walkers[0].step(self.walker_states[0]);
            self.walker_states[1] = self.walkers[1].step(self.walker_states[1]);
            self.buffers[0][self.buf_pos] = a;
            self.buffers[1][self.buf_pos] = b;
            out_left.set((a * (1. - wet_mix) + (aa * wet_mix)) * 0.5 * gain);
            out_right.set((b * (1. - wet_mix) + (bb * wet_mix)) * 0.5 * gain);
            self.buf_pos += 1;
            if self.buf_pos == self.buffers[0].len() {
                self.buf_pos = 0;
            }
        }
    }
}

lv2_descriptors!(JiglAgain);

#[cfg(test)]
mod tests {
    use super::{Walker, WalkerState};

    #[test]
    fn walker() {
        let w = Walker::new();
        println!("{:?}", w);
        let s = WalkerState::new_from_walker(&w);
        println!("{:?}", s);
        let mut next_s = w.step(s);
        println!("{:?}", next_s);
        assert_ne!(s, next_s);
        for _ in 1..w.acceleration_hold * 2 {
            next_s = w.step(next_s);
            println!("{:?}", next_s);
        }
    }
}
