use gilrs::{GamepadId, Gilrs};
use jack::{AudioOut, Client, Port};

const CLIENT_NAME: &str = "projet_u_phasor";

struct Phasor {
    out_port: Port<AudioOut>,
    sample_rate: f32,
    phase: f32,
    gilrs: Gilrs,
    gamepad_id: Option<GamepadId>,
    goal_frequency: f32,
    smooth_frequency: f32,
}

impl Phasor {
    fn new(client: &Client) -> Result<Self, Box<dyn std::error::Error>> {
        let out_port = client.register_port("phase_out", jack::AudioOut::default())?;
        let sample_rate = client.sample_rate() as f32;

        let gilrs = Gilrs::new()?;
        let gamepad_id = gilrs.gamepads().next().map(|(id, _)| id);

        Ok(Self {
            out_port,
            sample_rate,
            phase: 0.0,
            gilrs,
            gamepad_id,
            goal_frequency: 440.0,
            smooth_frequency: 440.0,
        })
    }
}

impl jack::ProcessHandler for Phasor {
    fn process(&mut self, _: &Client, ps: &jack::ProcessScope) -> jack::Control {
        const MIN_FREQUENCY: f32 = 40.0;
        const MAX_FREQUENCY: f32 = 880.0;

        let out_slice = self.out_port.as_mut_slice(ps);

        for sample in out_slice {
            *sample = self.phase;
            self.gilrs.next_event(); // Poll for events
            let gamepad = self.gamepad_id.map(|id| self.gilrs.gamepad(id));

            self.goal_frequency = {
                let joystick_in = gamepad
                    .and_then(|g| g.value(gilrs::Axis::LeftStickY).into())
                    .unwrap_or(0.0)
                    .clamp(-1.0, 1.0);

                ((joystick_in + 1.0) / 2.0) * (MAX_FREQUENCY - MIN_FREQUENCY) + MIN_FREQUENCY
            };

            self.smooth_frequency += (self.goal_frequency - self.smooth_frequency) * 0.001;

            let phase_increment_amount = self.goal_frequency / self.sample_rate;

            self.phase += phase_increment_amount;

            if self.phase >= 1.0 {
                self.phase = 0.0
            }
        }

        jack::Control::Continue
    }
}

fn main() {
    let (client, _status) = jack::Client::new(CLIENT_NAME, jack::ClientOptions::default())
        .expect("Failed to create JACK client");

    let phase_processor = Phasor::new(&client).expect("Failed to create Phasor");

    let active_client = client
        .activate_async((), phase_processor)
        .expect("Failed to activate client");

    println!("Phasor JACK client activated. Press Enter to quit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    if let Err(e) = active_client.deactivate() {
        eprintln!("Failed to deactivate client: {}", e);
    }
}
