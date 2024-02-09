use embassy_rp::gpio::{Output, Pin};
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

impl<'d, T: Pin> Switch<'d, T: Pin> {
    pub fn new(
        pin_0: &'d T,
        pin_1: &'d T,
        pin_2: &'d T,
        pin_3: &'d T,
        pin_4: &'d T,
        pin_5: &'d T,
    ) -> Self {
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
}
