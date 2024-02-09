#![no_std]
#![no_main]

use cyw43_pio::PioSpi;
use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::tcp::TcpSocket;
use embassy_net::{Config, Stack, StackResources};
use embassy_net::{IpAddress, IpEndpoint};
use embassy_rp::peripherals::{DMA_CH0, PIN_23, PIN_25, PIO0};
use embassy_rp::pio::Pio;
use embassy_rp::{bind_interrupts, gpio};
use embassy_time::Duration;
use embassy_time::Timer;
use gpio::{Level, Output};
use heapless::Vec;
use postcard::from_bytes;
use rust_mqtt::{
    client::{client::MqttClient, client_config::ClientConfig},
    utils::rng_generator::CountingRng,
};
use serde::{Deserialize, Serialize};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use crate::switch;

// use rust_mqtt::{
//     client::{client::MqttClient, client_config::ClientConfig},
//     packet::v5::reason_codes::ReasonCode,
//     utils::rng_generator::CountingRng,
// };

const WIFI_NETWORK: &str = "SilesianCloud-guest";
const WIFI_PASSWORD: &str = "T@jlandia123qwe";

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

#[derive(Serialize, Deserialize, Debug)]
struct Measurments {
    cotwo: u16,
    temp: f32,
    humdt: f32,
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

    let config = Config::dhcpv4(Default::default());
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
    while !stack.is_config_up() {
        Timer::after_millis(100).await;
    }
    info!("DHCP is now up!");

    let mut led = Output::new(p.PIN_25, Level::Low);
    let mut async_input = Input::new(p.PIN_16, Pull::None);

    let mut power_0 = Output::new(p.PIN_21, Level::Low);
    let mut power_1 = Output::new(p.PIN_20, Level::Low);
    let mut power_2 = Output::new(p.PIN_19, Level::Low);
    let mut power_3 = Output::new(p.PIN_18, Level::Low);
    let mut power_4 = Output::new(p.PIN_17, Level::Low);
    let mut power_5 = Output::new(p.PIN_16, Level::Low);

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    debug!("STARTING");

    let remote_endpoint = IpEndpoint::new(IpAddress::v4(192, 168, 1, 1), 1883);
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    debug!("CONNECT");
    socket.connect(remote_endpoint).await.unwrap();
    debug!("AFTER CONNECT");

    let mut config = ClientConfig::new(
        rust_mqtt::client::client_config::MqttVersion::MQTTv5,
        CountingRng(20000),
    );
    config.add_max_subscribe_qos(rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS1);
    config.add_client_id("cotwo-sensor");
    // config.add_username(USERNAME);
    // config.add_password(PASSWORD);
    config.max_packet_size = 100;
    let mut recv_buffer = [0; 80];
    let mut write_buffer = [0; 80];

    let mut client =
        MqttClient::<_, 5, _>::new(socket, &mut write_buffer, 80, &mut recv_buffer, 80, config);
    debug!("BROKER CONNECTING");
    client.connect_to_broker().await.unwrap();
    debug!("BROKER AFTER CONNECTING");
    let mut topic_names = Vec::<_, 2>::new();
    topic_names.push("switch_0").unwrap();
    topic_names.push("switch_1").unwrap();

    client.subscribe_to_topics(&topic_names).await.unwrap();
    Timer::after_millis(500).await;

    loop {
        // client
        //     .send_message(
        //         "test",
        //         b"hello2",
        //         rust_mqtt::packet::v5::publish_packet::QualityOfService::QoS0,
        //         true,
        //     )
        //     .await
        //     .unwrap();
        Timer::after_millis(500).await;

        match select(
            client.receive_message(),
            Timer::after(Duration::from_secs(2)),
        )
        .await
        {
            Either::First(msg) => {
                let (topic, message) = msg.unwrap();
                info!("topic: {}, message: {}", topic, message);

                let data: Measurments = from_bytes(message).unwrap();

                info!(
                    "Measurementy przyszly: {} {} {}",
                    data.cotwo, data.humdt, data.temp
                );
            }
            Either::Second(_timeout) => {
                info!("sending ping");
                client.send_ping().await.unwrap();
            }
        }

        // let (topic, message) = match client.receive_message().await {
        //     Ok(msg) => msg,
        //     Err(err) => {
        //         error!("ERROR OCCURED: {}", err);
        //         continue;
        //     }
        // };
        // info!("topic: {}, message: {}", topic, message);
    }

    // loop {
    //     match socket.write_all(data).await {
    //         Ok(()) => {}
    //         Err(e) => {
    //             warn!("write error: {:?}", e);
    //             break;
    //         }
    //     };
    // }
    power_0.set_high();
    power_1.set_high();
    power_2.set_high();
    power_3.set_high();
    power_4.set_high();
    power_5.set_high();
    Timer::after_secs(2).await;

    info!("done wait_for_high. Turn off LED");
    power_0.set_low();
    power_1.set_low();
    power_2.set_low();
    power_3.set_low();
    power_4.set_low();
    power_5.set_low();

    // Timer::after_secs(1).await;
}
