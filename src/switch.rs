use embassy_rp::gpio::Pin;
use embassy_rp::gpio::Output;
use embassy_time::Duration;

enum State {
    On,
    Off,
}


struct Timer {
    enabled: bool,
    duration: Duration,
}
struct Port<'d, T: Pin> {
    pin: Output<'d, T>,
    state: State,
    duration: Option<Timer>,
}

struct Switch<'d, T: Pin> {
    port_0: Port<'d, T>,
    port_1: Port<'d, T>,
    port_2: Port<'d, T>,
    port_3: Port<'d, T>,
    port_4: Port<'d, T>,
    port_5: Port<'d, T>,
}

struct PortCard {
    state: State,
    duration: Option<Timer>,
}

struct SwitchCard {
    port_0: PortCard,
    port_1: PortCard,
    port_2: PortCard,
    port_3: PortCard,
    port_4: PortCard,
    port_5: PortCard,
}

impl<'d, T: Pin> Switch<'d, T: Pin> {
    pub fn new(pin_0: T, pin_1: T, pin_2: T, pin_3: T, pin_4: T, pin_5: T) -> Self {
        Switch {
            port_0: Port {
                pin: pin_0,
                state: State::Off,
                duration: None,
            },
            port_1: Port {
                pin: pin_1,
                state: State::Off,
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

    pub fn apply(self) {
        self.port_0.pin.
    }
}
