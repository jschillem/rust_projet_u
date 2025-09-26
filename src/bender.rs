use gilrs::{GamepadId, Gilrs};
use jack::{AudioIn, AudioOut, Client, Port};

const CLIENT_NAME: &str = "projet_u_bender";

struct Phasor {
    in_port: Port<AudioIn>,
    out_port: Port<AudioOut>,
    gilrs: Gilrs,
    gamepad_id: Option<GamepadId>,
}

impl Phasor {
    fn new(client: &Client) -> Result<Self, Box<dyn std::error::Error>> {
        let in_port = client.register_port("phase_in", jack::AudioIn::default())?;
        let out_port = client.register_port("bent_out", jack::AudioOut::default())?;

        let gilrs = Gilrs::new()?;
        let gamepad_id = gilrs.gamepads().next().map(|(id, _)| id);

        Ok(Self {
            in_port,
            out_port,
            gilrs,
            gamepad_id,
        })
    }
}

impl jack::ProcessHandler for Phasor {
    fn process(&mut self, _: &Client, ps: &jack::ProcessScope) -> jack::Control {
        const MIN_FREQUENCY: f32 = 40.0;
        const MAX_FREQUENCY: f32 = 880.0;

        let in_slice = self.in_port.as_slice(ps);
        let out_slice = self.out_port.as_mut_slice(ps);

        for (in_sample, out_sample) in in_slice.iter().zip(out_slice) {
            self.gilrs.next_event(); // Poll for events
            let gamepad = self.gamepad_id.map(|id| self.gilrs.gamepad(id));

            let right_stick_x = gamepad
                .and_then(|g| g.value(gilrs::Axis::RightStickX).into())
                .unwrap_or(0.0)
                .clamp(-1.0, 1.0);

            let right_stick_y = gamepad
                .and_then(|g| g.value(gilrs::Axis::RightStickY).into())
                .unwrap_or(0.0)
                .clamp(-1.0, 1.0);

            // set x and y to be in range 0.05f to 0.95f
            let x = ((right_stick_x + 1.0) / 2.0) * (0.95 - 0.05) + 0.05;
            let y = ((right_stick_y + 1.0) / 2.0) * (0.95 - 0.05) + 0.05;

            if *in_sample < x {
                *out_sample = (y / x) * (*in_sample);
            } else {
                *out_sample = ((1.0 - y) / (1.0 - x)) * (*in_sample - x) + y;
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

    println!("Bender JACK client activated. Press Enter to quit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    if let Err(e) = active_client.deactivate() {
        eprintln!("Failed to deactivate client: {}", e);
    }
}
