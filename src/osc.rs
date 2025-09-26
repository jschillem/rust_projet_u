const CLIENT_NAME: &str = "projet_u_osc";

fn main() {
    let (client, _status) = jack::Client::new(CLIENT_NAME, jack::ClientOptions::default())
        .expect("Failed to create JACK client");

    let phase_in_port = client
        .register_port("phase_in", jack::AudioIn::default())
        .expect("Failed to create phase input port.");

    let mut out_port = client
        .register_port("out", jack::AudioOut::default())
        .expect("Failed to create output port");

    let on_process = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let phase = phase_in_port.as_slice(ps);
        let out = out_port.as_mut_slice(ps);
        // let sample_rate = client.sample_rate() as f32;

        for (sample, phase) in out.iter_mut().zip(phase) {
            // sine wave from phase
            *sample = (*phase * 2.0 * std::f32::consts::PI).sin() * 0.5;
        }

        jack::Control::Continue
    };

    let process_handler = jack::contrib::ClosureProcessHandler::new(on_process);

    let active_client = client
        .activate_async((), process_handler)
        .expect("Failed to activate client");

    println!("Osc JACK client activated. Press Enter to quit...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();

    if let Err(e) = active_client.deactivate() {
        eprintln!("Failed to deactivate client: {}", e);
    }
}
