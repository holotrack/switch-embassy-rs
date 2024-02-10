use defmt::debug;
use defmt::info;
use embassy_rp::gpio::AnyPin;
use embassy_rp::gpio::Output;

enum State {
    On,
    Off,
}

struct Timer {
    seconds: u32,
}
struct Port<'d> {
    pin: Output<'d, AnyPin>,
    state: State,
    duration: Option<Timer>,
}

pub struct Switch<'d> {
    port_0: Port<'d>,
    port_1: Port<'d>,
    port_2: Port<'d>,
    port_3: Port<'d>,
    port_4: Port<'d>,
    port_5: Port<'d>,
}

struct PortCard {
    state: State,
    duration: Option<Timer>,
}

pub struct SwitchCard {
    port_0: PortCard,
    port_1: PortCard,
    port_2: PortCard,
    port_3: PortCard,
    port_4: PortCard,
    port_5: PortCard,
}

impl<'d> Switch<'d> {
    pub fn new(
        pin_0: Output<'d, AnyPin>,
        pin_1: Output<'d, AnyPin>,
        pin_2: Output<'d, AnyPin>,
        pin_3: Output<'d, AnyPin>,
        pin_4: Output<'d, AnyPin>,
        pin_5: Output<'d, AnyPin>,
    ) -> Self {
        Switch {
            port_0: Port {
                pin: pin_0,
                state: State::On,
                duration: None,
            },
            port_1: Port {
                pin: pin_1,
                state: State::On,
                duration: None,
            },
            port_2: Port {
                pin: pin_2,
                state: State::Off,
                duration: None,
            },
            port_3: Port {
                pin: pin_3,
                state: State::Off,
                duration: None,
            },
            port_4: Port {
                pin: pin_4,
                state: State::Off,
                duration: None,
            },
            port_5: Port {
                pin: pin_5,
                state: State::Off,
                duration: None,
            },
        }
    }

    pub fn set_switch(mut self, switch: SwitchCard) {
        self.port_0.state = switch.port_0.state;
        self.port_0.duration = switch.port_0.duration;

        self.port_1.state = switch.port_1.state;
        self.port_1.duration = switch.port_1.duration;

        self.port_2.state = switch.port_2.state;
        self.port_2.duration = switch.port_2.duration;

        self.port_3.state = switch.port_3.state;
        self.port_3.duration = switch.port_3.duration;

        self.port_4.state = switch.port_4.state;
        self.port_4.duration = switch.port_4.duration;

        self.port_5.state = switch.port_5.state;
        self.port_5.duration = switch.port_5.duration;
    }

    fn apply_port(port: &mut Port<'d>) {
        debug!("APPLY PORT");
        match port {
            Port {
                state: State::On, ..
            } => {
                debug!("SET TO HIGH");
                port.pin.set_high();
            }
            Port {
                state: State::Off, ..
            } => {
                debug!("SET TO LOW");
                port.pin.set_low();
            }
        }

        // This need to be still properly implemented
        match port {
            Port { duration: None, .. } => info!("Duration not provided, ommiting "),
            Port {
                duration: Some(dur),
                ..
            } => info!("Duration provided: {}", dur.seconds),
        }
    }

    pub fn apply(&mut self) {
        debug!("APPLY PORT");

        Switch::apply_port(&mut self.port_0);
        Switch::apply_port(&mut self.port_1);
        Switch::apply_port(&mut self.port_2);
        Switch::apply_port(&mut self.port_3);
        Switch::apply_port(&mut self.port_4);
        Switch::apply_port(&mut self.port_5);
    }
}
