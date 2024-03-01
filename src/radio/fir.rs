//! FIR coefficient tables
//!
//! These are part of libad9361-iio and were generated by some bizarre process
//! that I don't want to replicate, so I'm just copying them.

pub struct FirConfig {
    pub decimation_factor: u32,
    pub fir_coefficients: &'static [i16],
}

impl FirConfig {
    const fn new<const DECIMATE: u32, const N: usize>(fir_coefficients: &'static [i16; N]) -> Self {
        Self {
            decimation_factor: DECIMATE,
            fir_coefficients,
        }
    }
}

pub const FIR_128_4: FirConfig = FirConfig::new::<4, 128>(&[
    -15, -27, -23, -6, 17, 33, 31, 9, -23, -47, -45, -13, 34, 69, 67, 21, -49, -102, -99, -32, 69,
    146, 143, 48, -96, -204, -200, -69, 129, 278, 275, 97, -170, -372, -371, -135, 222, 494, 497,
    187, -288, -654, -665, -258, 376, 875, 902, 363, -500, -1201, -1265, -530, 699, 1748, 1906,
    845, -1089, -2922, -3424, -1697, 2326, 7714, 12821, 15921, 15921, 12821, 7714, 2326, -1697,
    -3424, -2922, -1089, 845, 1906, 1748, 699, -530, -1265, -1201, -500, 363, 902, 875, 376, -258,
    -665, -654, -288, 187, 497, 494, 222, -135, -371, -372, -170, 97, 275, 278, 129, -69, -200,
    -204, -96, 48, 143, 146, 69, -32, -99, -102, -49, 21, 67, 69, 34, -13, -45, -47, -23, 9, 31,
    33, 17, -6, -23, -27, -15,
]);

pub const FIR_128_2: FirConfig = FirConfig::new::<2, 128>(&[
    -0, 0, 1, -0, -2, 0, 3, -0, -5, 0, 8, -0, -11, 0, 17, -0, -24, 0, 33, -0, -45, 0, 61, -0, -80,
    0, 104, -0, -134, 0, 169, -0, -213, 0, 264, -0, -327, 0, 401, -0, -489, 0, 595, -0, -724, 0,
    880, -0, -1075, 0, 1323, -0, -1652, 0, 2114, -0, -2819, 0, 4056, -0, -6883, 0, 20837, 32767,
    20837, 0, -6883, -0, 4056, 0, -2819, -0, 2114, 0, -1652, -0, 1323, 0, -1075, -0, 880, 0, -724,
    -0, 595, 0, -489, -0, 401, 0, -327, -0, 264, 0, -213, -0, 169, 0, -134, -0, 104, 0, -80, -0,
    61, 0, -45, -0, 33, 0, -24, -0, 17, 0, -11, -0, 8, 0, -5, -0, 3, 0, -2, -0, 1, 0, -0, 0,
]);

pub const FIR_96_2: FirConfig = FirConfig::new::<2, 96>(&[
    -4, 0, 8, -0, -14, 0, 23, -0, -36, 0, 52, -0, -75, 0, 104, -0, -140, 0, 186, -0, -243, 0, 314,
    -0, -400, 0, 505, -0, -634, 0, 793, -0, -993, 0, 1247, -0, -1585, 0, 2056, -0, -2773, 0, 4022,
    -0, -6862, 0, 20830, 32767, 20830, 0, -6862, -0, 4022, 0, -2773, -0, 2056, 0, -1585, -0, 1247,
    0, -993, -0, 793, 0, -634, -0, 505, 0, -400, -0, 314, 0, -243, -0, 186, 0, -140, -0, 104, 0,
    -75, -0, 52, 0, -36, -0, 23, 0, -14, -0, 8, 0, -4, 0,
]);

pub const FIR_64_2: FirConfig = FirConfig::new::<2, 64>(&[
    -58, 0, 83, -0, -127, 0, 185, -0, -262, 0, 361, -0, -488, 0, 648, -0, -853, 0, 1117, -0, -1466,
    0, 1954, -0, -2689, 0, 3960, -0, -6825, 0, 20818, 32767, 20818, 0, -6825, -0, 3960, 0, -2689,
    -0, 1954, 0, -1466, -0, 1117, 0, -853, -0, 648, 0, -488, -0, 361, 0, -262, -0, 185, 0, -127,
    -0, 83, 0, -58, 0,
]);
