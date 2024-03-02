use industrial_io;
use pluton::{self, };
use uom::si::u64::Frequency;

fn main() {
    let ctx = industrial_io::Context::from_uri("ip:pluto.local").unwrap();
    let mut radio = pluton::radio::Pluto::from_context(ctx).unwrap();
    let sample_rate = radio.get_sample_rate().unwrap();
    println!(
        "Current sample rate: {}",
        sample_rate.into_format_args(
            uom::si::frequency::hertz,
            uom::fmt::DisplayStyle::Description
        )
    );

	radio.set_sample_rate(Frequency::new::<uom::si::frequency::megahertz>(2)).unwrap();

    let sample_rate = radio.get_sample_rate().unwrap();
    println!(
        "Updated sample rate: {}",
        sample_rate.into_format_args(
            uom::si::frequency::hertz,
            uom::fmt::DisplayStyle::Description
        )
    );
}
