use industrial_io;
use pluton::{self};
use uom::si::u64::Frequency;

fn main() {
    let ctx = industrial_io::Context::from_uri("ip:pluto.local").unwrap();
    println!("Acquired IIO context");
    let mut radio = pluton::radio::Pluto::from_context(ctx).unwrap();
    let sample_rate = radio.get_sample_rate().expect("Failed to get sample rate");
    println!(
        "Current sample rate: {}",
        sample_rate.into_format_args(
            uom::si::frequency::hertz,
            uom::fmt::DisplayStyle::Description
        )
    );

    let rx_carrier = radio
        .get_rx_carrier_freq()
        .expect("Failed to get RX carrier frequency");
    println!(
        "Current RX carrier frequency: {}",
        rx_carrier.into_format_args(
            uom::si::frequency::hertz,
            uom::fmt::DisplayStyle::Description
        )
    );
    let tx_carrier = radio
        .get_tx_carrier_freq()
        .expect("Failed to get TX carrier frequency");
    println!(
        "Current TX carrier frequency: {}",
        tx_carrier.into_format_args(
            uom::si::frequency::hertz,
            uom::fmt::DisplayStyle::Description
        )
    );
    let gain_control_mode = radio
        .get_gain_control_mode()
        .expect("Failed to get gain control mode");
    println!("Current gain control mode: {}", gain_control_mode);
    let rx_hw_gain = radio
        .get_rx_hardware_gain()
        .expect("Failed to get RX hardware gain");
    println!("Current RX gain: {}", rx_hw_gain);
    let tx_hw_gain = radio
        .get_tx_hardware_gain()
        .expect("Failed to get TX hardware gain");
    println!("Current TX gain: {}", tx_hw_gain);
    let rx_bandwidth = radio
        .get_rx_rf_bandwidth()
        .expect("Failed to get RX bandwidth");
    println!(
        "Current RX bandwidth: {}",
        rx_bandwidth.into_format_args(
            uom::si::frequency::hertz,
            uom::fmt::DisplayStyle::Description
        )
    );
    let tx_bandwidth = radio
        .get_tx_rf_bandwidth()
        .expect("Failed to get TX bandwidth");
    println!(
        "Current TX bandwidth: {}",
        tx_bandwidth.into_format_args(
            uom::si::frequency::hertz,
            uom::fmt::DisplayStyle::Description
        )
    );
}
