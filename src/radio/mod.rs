use std::fmt::Write;

use industrial_io as iio;
use uom::si::{
    frequency::{hertz, kilohertz},
    u64::*,
};

mod fir;
use crate::{error::Error, Result};

/// Wrapper for interfacing with the ADALM-PLUTO from Analog Devices.
pub struct Pluto {
    phy_device: iio::Device,
}

impl Pluto {
    pub fn from_context(context: iio::Context) -> Result<Self> {
        let phy_device = Self::find_device(&context, "ad9361-phy")?;
        Ok(Self { phy_device })
    }

    pub fn get_sample_rate(&self) -> Result<Frequency> {
        let voltage0 = Self::find_channel(&self.phy_device, "voltage0", false)?;
        let sample_rate = voltage0.attr_read_int("sampling_frequency")?;
        let sample_rate = u64::try_from(sample_rate)?;

        Ok(Frequency::new::<hertz>(sample_rate))
    }

    pub fn set_sample_rate(&mut self, rate: Frequency) -> Result<()> {
        if rate < Frequency::new::<kilohertz>(521) {
            Err(Error::InvalidSampleRate)
        } else {
            // dec: decimation factor
            // taps: number of FIR coefficients
            // fir: list of FIR coefficients
            let fir_config = if rate <= Frequency::new::<hertz>(20000000) {
                &fir::FIR_128_4
            } else if rate <= Frequency::new::<hertz>(40000000) {
                &fir::FIR_128_2
            } else if rate <= Frequency::new::<hertz>(53333333) {
                &fir::FIR_96_2
            } else {
                &fir::FIR_64_2
            };

            let dec = fir_config.decimation_factor;
            let fir = fir_config.fir_coefficients;

            let current_sample_rate = self.get_sample_rate()?;
            let voltage0 = Self::find_channel(&self.phy_device, "voltage0", true)?;
            let fir_enabled = self.is_tx_fir_enabled()?;

            if fir_enabled {
                if current_sample_rate <= Frequency::new::<hertz>(25000000 / 12) {
                    voltage0.attr_write_int("sampling_frequency", 3000000)?;
                }

                self.set_tx_fir_enabled(false)?;
            }

            let mut fir_config = String::new();
            writeln!(&mut fir_config, "RX 3 GAIN -6 DEC {dec}")?;
            writeln!(&mut fir_config, "TX 3 GAIN 0 INT {dec}")?;

            for tap in fir {
                writeln!(&mut fir_config, "{0}/{0}", tap)?;
            }
            writeln!(&mut fir_config)?;

            self.phy_device
                .attr_write_str("filter_fir_config", &fir_config)?;

            if rate <= Frequency::new::<hertz>(25000000 / 12) {
                let tx_path_rates = self.phy_device.attr_read_str("tx_path_rates")?;
                let TxPathRates {
                    dac_sample_rate,
                    tx_sample_rate,
                    ..
                } = parse_tx_path_rates(&tx_path_rates)?;
                let max = (dac_sample_rate.get::<hertz>() / tx_sample_rate.get::<hertz>()) * 16;
                if max < fir.len() as u64 {
                    voltage0.attr_write_int("sampling_frequency", 3000000)?;
                }

                self.set_tx_fir_enabled(true)?;
                voltage0.attr_write_int("sampling_frequency", rate.get::<hertz>() as i64)?;
            } else {
                voltage0.attr_write_int("sampling_frequency", rate.get::<hertz>() as i64)?;
                self.set_tx_fir_enabled(true)?;
            }

            Ok(())
        }
    }

    fn is_tx_fir_enabled(&self) -> Result<bool> {
        self.phy_device
            .attr_read_bool("in_out_voltage_filter_fir_en")
            .or_else(|_| {
                Self::find_channel(&self.phy_device, "out", false)
                    .and_then(|c| Ok(c.attr_read_bool("voltage_filter_fir_en")?))
            })
    }

    fn set_tx_fir_enabled(&mut self, enable: bool) -> Result<()> {
        self.phy_device
            .attr_write_bool("in_out_voltage_filter_fir_en", enable)
            .or_else(|_| {
                Self::find_channel(&self.phy_device, "out", false)
                    .and_then(|c| Ok(c.attr_write_bool("voltage_filter_fir_en", enable)?))
            })
    }

    fn find_channel<'a>(
        device: &'a iio::Device,
        name: &'static str,
        is_output: bool,
    ) -> Result<iio::Channel> {
        device
            .find_channel(name, is_output)
            .ok_or(Error::CantFindChannel(name))
    }

    fn find_device<'a>(context: &'a iio::Context, name: &'static str) -> Result<iio::Device> {
        context.find_device(name).ok_or(Error::CantFindDevice(name))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct TxPathRates {
    tx_sample_rate: Frequency,
    hb1_filter_rate: Frequency,
    hb2_filter_rate: Frequency,
    hb3_filter_rate: Frequency,
    dac_sample_rate: Frequency,
    baseband_pll_freq: Frequency,
}

use nom::{
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{map_res, recognize},
    multi::{many0, many1},
    sequence::terminated,
    IResult, Parser,
};

fn decimal64(input: &str) -> IResult<&str, u64> {
    map_res(recognize(many1(one_of("0123456789"))), |x: &str| {
        u64::from_str_radix(x, 10)
    })
    .parse(input)
}

fn parse_tx_path_rates(input: &str) -> Result<TxPathRates> {
    let (_, tx_path_rates) = nom::combinator::all_consuming(move |input| {
        let (input, _) = tag("BBPLL:")(input)?;
        let (input, baseband_pll_freq) = decimal64(input)?;
        let (input, _) = tag(" ")(input)?;

        let (input, _) = tag("DAC:")(input)?;
        let (input, dac_sample_rate) = decimal64(input)?;
        let (input, _) = tag(" ")(input)?;

        let (input, _) = tag("T2:")(input)?;
        let (input, hb3_filter_rate) = decimal64(input)?;
        let (input, _) = tag(" ")(input)?;

        let (input, _) = tag("T1:")(input)?;
        let (input, hb2_filter_rate) = decimal64(input)?;
        let (input, _) = tag(" ")(input)?;

        let (input, _) = tag("TF:")(input)?;
        let (input, hb1_filter_rate) = decimal64(input)?;
        let (input, _) = tag(" ")(input)?;

        let (input, _) = tag("TXSAMP:")(input)?;
        let (input, tx_sample_rate) = decimal64(input)?;

        Ok((
            input,
            TxPathRates {
                tx_sample_rate: Frequency::new::<hertz>(tx_sample_rate),
                hb1_filter_rate: Frequency::new::<hertz>(hb1_filter_rate),
                hb2_filter_rate: Frequency::new::<hertz>(hb2_filter_rate),
                hb3_filter_rate: Frequency::new::<hertz>(hb3_filter_rate),
                dac_sample_rate: Frequency::new::<hertz>(dac_sample_rate),
                baseband_pll_freq: Frequency::new::<hertz>(baseband_pll_freq),
            },
        ))
    })(input)
    .map_err(|_| Error::ParsingError)?;

    Ok(tx_path_rates)
}

#[cfg(test)]
mod tests {
    use uom::si::{frequency::hertz, u64::Frequency};

    use crate::radio::TxPathRates;

    use super::parse_tx_path_rates;

    #[test]
    fn parsing_tx_path_rates_works() {
        let path_rates = parse_tx_path_rates(
            "BBPLL:1024000006 DAC:128000000 T2:64000000 T1:32000000 TF:16000000 TXSAMP:4000000",
        )
        .unwrap();
        assert_eq!(
            path_rates,
            TxPathRates {
                tx_sample_rate: Frequency::new::<hertz>(4000000),
                hb1_filter_rate: Frequency::new::<hertz>(16000000),
                hb2_filter_rate: Frequency::new::<hertz>(32000000),
                hb3_filter_rate: Frequency::new::<hertz>(64000000),
                dac_sample_rate: Frequency::new::<hertz>(128000000),
                baseband_pll_freq: Frequency::new::<hertz>(1024000006)
            }
        );
    }
}
