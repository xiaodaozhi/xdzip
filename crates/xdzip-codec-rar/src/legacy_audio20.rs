//! RAR20/26's interleaved predictive audio mode, not a post-LZ filter.
//! Math follows the frozen RAR20-26 facts; no third-party implementation read.

use xdzip_codec_api::payload::DecodeError;

#[derive(Default)]
struct Channel {
    previous: u8,
    delta: i32,
    differences: [i32; 4],
    coefficients: [i32; 5],
    errors: [u32; 11],
    // Only the phase modulo32 affects adaptation. Host work/output counters
    // independently retain total sizes, including files beyond u32 samples.
    phase: u8,
}

/// All channel models and their shared previous-channel difference survive
/// table and mode changes. Only a non-solid generation reset clears them.
pub(crate) struct Audio20 {
    models: [Channel; 4],
    channels: usize,
    current: usize,
    shared_delta: i32,
}
impl Default for Audio20 {
    fn default() -> Self {
        Self {
            models: std::array::from_fn(|_| Channel::default()),
            channels: 1,
            current: 0,
            shared_delta: 0,
        }
    }
}
impl Audio20 {
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn configure(&mut self, channels: usize) -> Result<(), DecodeError> {
        if !(1..=4).contains(&channels) {
            return Err(DecodeError::Integrity);
        }
        self.channels = channels;
        if self.current >= channels {
            self.current = 0;
        }
        Ok(())
    }
    pub fn channel(&self) -> usize {
        self.current
    }
    pub fn sample(&mut self, residual: u8) -> u8 {
        let channel = &mut self.models[self.current];
        channel.differences = [
            channel.delta,
            channel.delta - channel.differences[0],
            channel.differences[1],
            channel.differences[2],
        ];
        let features = [
            channel.differences[0],
            channel.differences[1],
            channel.differences[2],
            channel.differences[3],
            self.shared_delta,
        ];
        // Signed features are at most255, coefficients -17..16, and p<=255.
        // This fixed sum fits i32. div_euclid preserves floor for negative sums.
        let prediction = channel.coefficients.iter().zip(&features).fold(
            8 * i32::from(channel.previous),
            |sum, (&coefficient, &feature)| sum + coefficient * feature,
        );
        let value = (prediction.div_euclid(8) as u8).wrapping_sub(residual);
        let error = 8 * i32::from(residual as i8);
        channel.errors[0] += error.unsigned_abs();
        for (index, &feature) in features.iter().enumerate() {
            channel.errors[1 + 2 * index] += (error - feature).unsigned_abs();
            channel.errors[2 + 2 * index] += (error + feature).unsigned_abs();
        }
        channel.delta = i32::from(value.wrapping_sub(channel.previous) as i8);
        self.shared_delta = channel.delta;
        channel.previous = value;
        channel.phase = (channel.phase + 1) & 31;
        if channel.phase == 0 {
            // Strict improvement selects the first index on equal minima.
            let mut best = 0;
            for index in 1..channel.errors.len() {
                if channel.errors[index] < channel.errors[best] {
                    best = index;
                }
            }
            channel.errors.fill(0);
            if best != 0 {
                let coefficient = &mut channel.coefficients[(best - 1) / 2];
                if best & 1 != 0 {
                    if *coefficient >= -16 {
                        *coefficient -= 1;
                    }
                } else if *coefficient < 16 {
                    *coefficient += 1;
                }
            }
        }
        self.current = (self.current + 1) % self.channels;
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_prediction_uses_floor_and_global_delta_crosses_channels() {
        let mut audio = Audio20::default();
        audio.models[0].delta = -1;
        audio.models[0].coefficients[0] = 1;
        assert_eq!(audio.sample(0), 255);
        audio.reset();
        audio.configure(2).unwrap();
        audio.models[1].coefficients[4] = 1;
        assert_eq!(audio.sample(1), 255);
        assert_eq!(audio.sample(0), 255);
    }

    #[test]
    fn adaptation_is_after_the_32nd_sample_and_ties_choose_lowest_index() {
        let mut audio = Audio20::default();
        audio.models[0].previous = 100;
        audio.models[0].delta = 5;
        audio.shared_delta = 9;
        audio.sample(1);
        assert_eq!(audio.models[0].coefficients, [0; 5]);
        audio.reset();
        let channel = &mut audio.models[0];
        channel.phase = 31;
        channel.errors.fill(10000);
        channel.errors[1] = 0;
        channel.errors[2] = 0;
        audio.sample(0);
        assert_eq!(audio.models[0].coefficients, [-1, 0, 0, 0, 0]);
        assert_eq!(audio.models[0].errors, [0; 11]);
    }

    #[test]
    fn coefficient_minus17_and_plus16_are_reachable_and_stable_bounds() {
        for (start, best, expected) in [(-16, 1, -17), (-17, 1, -17), (15, 2, 16), (16, 2, 16)] {
            let mut audio = Audio20::default();
            let channel = &mut audio.models[0];
            channel.phase = 31;
            channel.coefficients[0] = start;
            channel.errors.fill(10000);
            channel.errors[best] = 0;
            audio.sample(0);
            assert_eq!(audio.models[0].coefficients[0], expected);
        }
    }

    #[test]
    fn new_channel_count_preserves_models_and_only_repairs_invalid_current() {
        let mut audio = Audio20::default();
        audio.configure(4).unwrap();
        audio.sample(1);
        audio.sample(2);
        assert_eq!(audio.channel(), 2);
        audio.configure(3).unwrap();
        assert_eq!(audio.channel(), 2);
        audio.configure(2).unwrap();
        assert_eq!(audio.channel(), 0);
        assert_eq!(audio.models[0].previous, 255);
        assert_eq!(audio.models[1].previous, 254);
        assert_eq!(audio.shared_delta, -2);
        assert!(audio.configure(0).is_err());
        audio.reset();
        assert_eq!(audio.channel(), 0);
        assert_eq!(audio.shared_delta, 0);
    }
}

#[cfg(test)]
mod independent_math_tests {
    use super::*;
    #[test]
    fn python_audio_channels_crosscheck_update_boundaries() {
        check(
            1,
            31,
            "ba28c2ea3f1bba7013dc45352c575ddecc22ee323645267a741b6f27c38d8c88",
        );
        check(
            1,
            32,
            "7e108a71710f9777fc215e3e67a3d4a38a594b1a6cd6cb649f8b7f289bd770a9",
        );
        check(
            1,
            33,
            "74951fd0640f0e90f23eadc9a11f9b81cf7f5bb285a4320ce0d04c5e450d3034",
        );
        check(
            1,
            4097,
            "98df5905ab204f73a82593ad5ba0dbde42fbbd85b14953abcbaf7baadd488832",
        );
        check(
            2,
            31,
            "34190fc382287222343a046a4412038c482366ec530ef02c9b61b294ed5c57fd",
        );
        check(
            2,
            32,
            "117eba5fc8686b09cde978bb423463b1cf6c021aa2a2e44f75f8deeec88afa96",
        );
        check(
            2,
            33,
            "867852babcf4d9a7d1040feabd2b39cd00525c79ca14abffae53d328f38790db",
        );
        check(
            2,
            4097,
            "b601a246019e2d1289676bb65b7a2a7c990e13d182abba363ab2e29613b6f7af",
        );
        check(
            3,
            31,
            "0edd274fe0cd881e25249232510a6845736cd908832eea21e61f951073d08bf5",
        );
        check(
            3,
            32,
            "d4c6d9dd8b6b24479d4e31e5227eaa2036436be78287d6d0570332cdaf7b322c",
        );
        check(
            3,
            33,
            "ac077101e10053910fd4e911a18b52ce7052f7e7eeb1772cb155e26ff9cf7ac2",
        );
        check(
            3,
            4097,
            "b15c5c230f77d43fe0088e1d5b4103f82639b616e74452541bb61f15b57a96bf",
        );
        check(
            4,
            31,
            "5a93b0f5fc53b1b7b59ea7ab037dbdfbad28ed5621cf1319a0059b4c35cac813",
        );
        check(
            4,
            32,
            "2240044fb16bc2fe1b33c021a212c43367c743885479a0b1d4e0b273db8f5946",
        );
        check(
            4,
            33,
            "a7a503d3add871b92746bca5d540d788a295092f2c4092ef200794394887da68",
        );
        check(
            4,
            4097,
            "0f2bdbf2063b590b37681005da9d2d9c1481383f9fbb4a20411a5f9e12b4a444",
        );
    }
    fn check(channels: usize, count: usize, expected: &str) {
        let mut audio = Audio20::default();
        audio.configure(channels).unwrap();
        let mut hash = xdzip_crypto::SecretSha256::new();
        for i in 0..count {
            let byte = audio.sample((i * 37 + i / 3 + 17) as u8);
            hash.update(&[byte]);
        }
        let actual: String = hash.finish().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(actual, expected, "channels={channels} count={count}");
    }
}
