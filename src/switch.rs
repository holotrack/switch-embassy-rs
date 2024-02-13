use defmt::debug;
use defmt::error;
use defmt::info;
use embassy_rp::gpio::AnyPin;
use embassy_rp::gpio::Output;
use heapless::Vec;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]

pub enum State {
    On,
    Off,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]

struct Timer {
    seconds: u32,
}
struct Port<'d> {
    pin: &'d mut Output<'d, AnyPin>,
    state: State,
    duration: Option<Timer>,
}

pub struct Switch<'d, const N: usize>(Vec<Port<'d>, N>);

impl<'d, const N: usize> Switch<'d, N> {
    pub fn new(pins: &'d mut Vec<Output<'d, AnyPin>, N>) -> Self {
        Self(
            pins.iter_mut()
                .map(|x| Port {
                    pin: x,
                    state: State::Off,
                    duration: None,
                })
                .collect(),
        )
    }
    // pub fn set_switch(&mut self, switch: SwitchCard) {
    //     self.port_4.state = switch.port_4.state;
    //     self.port_4.duration = switch.port_4.duration;
    // }

    pub fn apply(&mut self) {
        self.0
            .iter_mut()
            .enumerate()
            .for_each(|(index, port)| match port.state {
                State::On => {
                    info!("SET PORT {} ON", index);
                    port.pin.set_low();
                }
                State::Off => {
                    info!("SET PORT {} OFF", index);
                    port.pin.set_high();
                }
            })
    }

    pub fn set_port(&mut self, card: PortCard) {
        info!("Setting up port {}", card.port);
        match self.0.get_mut(card.port) {
            Some(p) => {
                p.state = card.state;
                p.duration = card.duration;
            }
            None => {
                error!("Not such port number: {}", card.port);
                return;
            }
        }
        self.apply_port(card.port);
    }

    fn apply_port(&mut self, port: usize) {
        match self.0.get_mut(port) {
            Some(p) => match p.state {
                State::On => {
                    info!("Turn port {} ON", port);
                    self.0[port].pin.set_low();
                }
                State::Off => {
                    info!("Turn port {} OFF", port);
                    self.0[port].pin.set_high();
                }
            },
            None => error!("Not such port number: {}", port),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PortCard {
    pub port: usize,
    pub state: State,
    pub duration: Option<Timer>,
}
