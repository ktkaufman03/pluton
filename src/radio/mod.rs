use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use industrial_io as iio;
use uom::si::{
    frequency::{hertz, kilohertz},
    u64::*,
};

mod fir;
use crate::{error::Error, Result};

pub enum GainControlMode {
    SlowAttack,
    FastAttack,
    Manual,
}

impl FromStr for GainControlMode {
    type Err = Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "slow_attack" => Ok(Self::SlowAttack),
            "fast_attack" => Ok(Self::FastAttack),
            "manual" => Ok(Self::Manual),
            x => Err(Error::InvalidGainControlMode(x.to_string())),
        }
    }
}

impl Display for GainControlMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::SlowAttack => f.write_str("slow_attack"),
            Self::FastAttack => f.write_str("fast_attack"),
            Self::Manual => f.write_str("manual"),
        }
    }
}

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
        self.read_chan_freq_attr("voltage0", false, "sampling_frequency")
    }

    pub fn set_sample_rate(&mut self, rate: Frequency) -> Result<()> {
        if rate < Frequency::new::<kilohertz>(521) {
            Err(Error::InvalidSampleRate)
        } else {
            // dec: decimation factor
            // taps: number of FIR coefficients
            // fir: list of FIR coefficients
            let &FirConfig {
                decimation_factor,
                fir_coefficients,
            } = if rate <= Frequency::new::<hertz>(20000000) {
                &fir::FIR_128_4
            } else if rate <= Frequency::new::<hertz>(40000000) {
                &fir::FIR_128_2
            } else if rate <= Frequency::new::<hertz>(53333333) {
                &fir::FIR_96_2
            } else {
                &fir::FIR_64_2
            };

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
            writeln!(&mut fir_config, "RX 3 GAIN -6 DEC {decimation_factor}")?;
            writeln!(&mut fir_config, "TX 3 GAIN 0 INT {decimation_factor}")?;

            for tap in fir_coefficients {
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
                if max < fir_coefficients.len() as u64 {
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

    pub fn get_rx_carrier_freq(&self) -> Result<Frequency> {
        self.read_chan_freq_attr("altvoltage0", true, "frequency")
    }

    pub fn set_rx_carrier_freq(&mut self, freq: Frequency) -> Result<()> {
        let altvoltage0 = Self::find_channel(&self.phy_device, "altvoltage0", true)?;
        altvoltage0.attr_write_int("frequency", freq.value as i64)?;
        Ok(())
    }

    pub fn get_tx_carrier_freq(&self) -> Result<Frequency> {
        self.read_chan_freq_attr("altvoltage1", true, "frequency")
    }

    pub fn set_tx_carrier_freq(&mut self, freq: Frequency) -> Result<()> {
        let altvoltage1 = Self::find_channel(&self.phy_device, "altvoltage1", true)?;
        altvoltage1.attr_write_int("frequency", freq.value as i64)?;
        Ok(())
    }

    pub fn get_gain_control_mode(&self) -> Result<GainControlMode> {
        Self::find_channel(&self.phy_device, "voltage0", false)
            .and_then(|c| c.attr_read_str("gain_control_mode").map_err(Error::from))
            .and_then(|s| GainControlMode::from_str(s.as_str()))
    }

    pub fn set_gain_control_mode(&mut self, mode: GainControlMode) -> Result<()> {
        Self::find_channel(&self.phy_device, "voltage0", false).and_then(|c| {
            c.attr_write_str("gain_control_mode", mode.to_string().as_str())
                .map_err(Error::from)
        })
    }

    pub fn get_rx_hardware_gain(&self) -> Result<f64> {
        Self::find_channel(&self.phy_device, "voltage0", false)
            .and_then(|c| c.attr_read_float("hardwaregain").map_err(Error::from))
    }

    pub fn set_rx_hardware_gain(&mut self, gain: f64) -> Result<()> {
        Self::find_channel(&self.phy_device, "voltage0", false).and_then(|c| {
            c.attr_write_float("hardwaregain", gain)
                .map_err(Error::from)
        })
    }

    pub fn get_tx_hardware_gain(&self) -> Result<f64> {
        Self::find_channel(&self.phy_device, "voltage0", true)
            .and_then(|c| c.attr_read_float("hardwaregain").map_err(Error::from))
    }

    pub fn set_tx_hardware_gain(&mut self, gain: f64) -> Result<()> {
        Self::find_channel(&self.phy_device, "voltage0", true).and_then(|c| {
            c.attr_write_float("hardwaregain", gain)
                .map_err(Error::from)
        })
    }

    pub fn get_rx_rf_bandwidth(&self) -> Result<Frequency> {
        self.read_chan_freq_attr("voltage0", false, "rf_bandwidth")
    }

    pub fn set_rx_rf_bandwidth(&mut self, bandwidth: Frequency) -> Result<()> {
        Self::find_channel(&self.phy_device, "voltage0", false).and_then(|c| {
            c.attr_write_int("rf_bandwidth", bandwidth.value as i64)
                .map_err(Error::from)
        })
    }

    pub fn get_tx_rf_bandwidth(&self) -> Result<Frequency> {
        self.read_chan_freq_attr("voltage0", true, "rf_bandwidth")
    }

    pub fn set_tx_rf_bandwidth(&mut self, bandwidth: Frequency) -> Result<()> {
        Self::find_channel(&self.phy_device, "voltage0", true).and_then(|c| {
            c.attr_write_int("rf_bandwidth", bandwidth.value as i64)
                .map_err(Error::from)
        })
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

    fn read_chan_freq_attr(
        &self,
        channel: &'static str,
        is_output: bool,
        attr: &'static str,
    ) -> Result<Frequency> {
        let altvoltage0 = Self::find_channel(&self.phy_device, channel, is_output)?;

        let frequency = altvoltage0.attr_read_int(attr)?;
        let sample_rate = u64::try_from(frequency)?;

        Ok(Frequency::new::<hertz>(sample_rate))
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

use self::fir::FirConfig;

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
