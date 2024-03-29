#![no_std]
#![no_main]

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;

use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Stack, StackResources};

use embassy_rp::gpio::{AnyPin, Pin};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_rp::pio::Pio;
use embassy_rp::{bind_interrupts, gpio};
use embassy_time::Duration;

use embedded_io_async::Write;
use gpio::{Level, Output};
use heapless::Vec;

use static_cell::StaticCell;
use switch_embassy_rs::switch::{Message, Switch};
use {defmt_rtt as _, panic_probe as _};

const WIFI_NETWORK: &str = "SilesianCloud-guest";
const WIFI_PASSWORD: &str = "T@jlandia123qwe";

const SOCKETS_AMMOUNT: usize = 6;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn wifi_task(
    runner: cyw43::Runner<
        'static,
        Output<'static, PIN_23>,
        PioSpi<'static, PIN_25, PIO0, 0, DMA_CH0>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // WiFi

    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");

    let pwr = Output::new(p.PIN_23, Level::Low);
    let cs = Output::new(p.PIN_25, Level::High);
    let mut pio = Pio::new(p.PIO0, Irqs);
    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        pio.irq0,
        cs,
        p.PIN_24,
        p.PIN_29,
        p.DMA_CH0,
    );

    static STATE: StaticCell<cyw43::State> = StaticCell::new();
    let state = STATE.init(cyw43::State::new());
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(wifi_task(runner)));

    control.init(clm).await;
    control
        .set_power_management(cyw43::PowerManagementMode::PowerSave)
        .await;

    let mut dhcp = embassy_net::DhcpConfig::default();
    dhcp.hostname = Some(heapless::String::try_from("switch-0").unwrap());
    let config = Config::dhcpv4(dhcp);
    //let config = embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
    //    address: Ipv4Cidr::new(Ipv4Address::new(192, 168, 69, 2), 24),
    //    dns_servers: Vec::new(),
    //    gateway: Some(Ipv4Address::new(192, 168, 69, 1)),
    //});

    // Generate random seed
    let seed = 0x0123_4567_89ab_cdef; // chosen by fair dice roll. guarenteed to be random.

    // Init network stack
    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<2>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        config,
        RESOURCES.init(StackResources::<2>::new()),
        seed,
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    loop {
        //control.join_open(WIFI_NETWORK).await;
        match control.join_wpa2(WIFI_NETWORK, WIFI_PASSWORD).await {
            Ok(_) => break,
            Err(err) => {
                info!("join failed with status={}", err.status);
            }
        }
    }

    // Wait for DHCP, not necessary when using static IP
    info!("waiting for DHCP...");
    stack.wait_config_up().await;
    info!("DHCP is now up!");

    //We need to .degrade() to have AnyPin type becuse its concrete type not trait. And we can not pass trait to generic type on embassy/embedded
    let power_0 = Output::new(p.PIN_21.degrade(), Level::Low);
    let power_1 = Output::new(p.PIN_20.degrade(), Level::Low);
    let power_2 = Output::new(p.PIN_19.degrade(), Level::Low);
    let power_3 = Output::new(p.PIN_18.degrade(), Level::Low);
    let power_4 = Output::new(p.PIN_17.degrade(), Level::Low);
    let power_5 = Output::new(p.PIN_16.degrade(), Level::Low);

    static SWITCH: StaticCell<Switch<SOCKETS_AMMOUNT>> = StaticCell::new();
    static POWER_SOCKETS: StaticCell<Vec<Output<'_, AnyPin>, 6>> = StaticCell::new();
    let power_sockets = POWER_SOCKETS.init(Vec::<Output<'_, AnyPin>, 6>::new());

    let _ = power_sockets.push(power_0);
    let _ = power_sockets.push(power_1);
    let _ = power_sockets.push(power_2);
    let _ = power_sockets.push(power_3);
    let _ = power_sockets.push(power_4);
    let _ = power_sockets.push(power_5);

    let switch = SWITCH.init(Switch::new(power_sockets));
    switch.apply();

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut data = [0u8; 1024];

    loop {
        debug!("SOCKET IN MAIN");
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        debug!("AFTER SOCKET IN MAIN");

        socket.set_timeout(Some(Duration::from_secs(10)));
        debug!("AFTER TIMEOUT IN MAIN");

        debug!("BEFORE SOCKET.ACCEPT SET IN MAIN");

        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }
        info!("Received connection from {:?}", socket.remote_endpoint());

        loop {
            debug!("READ");
            match socket.read(&mut data).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(_) => {
                    debug!("START READ");
                    debug!("Read data: {:?}", data);

                    let message: Message = match postcard::from_bytes(&data) {
                        Ok(msg) => msg,
                        Err(_) => {
                            error!("Serde from_bytes error probably recived wrong packet from someone else than controller");
                            break;
                        }
                    };

                    match message {
                        Message::SetPort(card) => {
                            switch.set_port(card);
                            // switch.apply();
                            info!("Port setted");
                        }
                        Message::GetPortStatus(card) => {
                            let status = Message::GetPortStatus(switch.get_port(card.unwrap()));
                            let status_slice = postcard::to_slice(&status, &mut data).unwrap();

                            debug!("Status slice: {:?}", status_slice);

                            debug!("WRITE ALL BEFORE");

                            match socket.write_all(status_slice).await {
                                Ok(()) => {
                                    info!("Status sent");
                                }
                                Err(e) => {
                                    error!("write error: {:?}", e);
                                    break;
                                }
                            };
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to receive data: {}", e);
                    break;
                }
            }
        }
    }
}
