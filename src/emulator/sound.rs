pub mod flags;

use flags::*;

/*
    Name Addr 7654 3210 Function
    -----------------------------------------------------------------
        Square 1
    NR10 FF10 -PPP NSSS Sweep period, negate, shift
    NR11 FF11 DDLL LLLL Duty, Length load (64-L)
    NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
    NR13 FF13 FFFF FFFF Frequency LSB
    NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB

        Square 2
        FF15 ---- ---- Not used
    NR21 FF16 DDLL LLLL Duty, Length load (64-L)
    NR22 FF17 VVVV APPP Starting volume, Envelope add mode, period
    NR23 FF18 FFFF FFFF Frequency LSB
    NR24 FF19 TL-- -FFF Trigger, Length enable, Frequency MSB

        Wave
    NR30 FF1A E--- ---- DAC power
    NR31 FF1B LLLL LLLL Length load (256-L)
    NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
    NR33 FF1D FFFF FFFF Frequency LSB
    NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB

        Noise
        FF1F ---- ---- Not used
    NR41 FF20 --LL LLLL Length load (64-L)
    NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
    NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
    NR44 FF23 TL-- ---- Trigger, Length enable

        Control/Status
    NR50 FF24 ALLL BRRR Vin L enable, Left vol, Vin R enable, Right vol
    NR51 FF25 NW21 NW21 Left enables, Right enables
    NR52 FF26 P--- NW21 Power control/status, Channel length statuses

        Not used
        FF27 ---- ----
        .... ---- ----
        FF2F ---- ----

        Wave Table
        FF30 0000 1111 Samples 0 and 1
        ....
        FF3F 0000 1111 Samples 30 and 31
*/

// NR10 FF10 -PPP NSSS Sweep period, negate, shift
// NR11 FF11 DDLL LLLL Duty, Length load (64-L)
// NR12 FF12 VVVV APPP Starting volume, Envelope add mode, period
// NR13 FF13 FFFF FFFF Frequency LSB
// NR14 FF14 TL-- -FFF Trigger, Length enable, Frequency MSB
#[allow(dead_code)]
#[derive(Default)]
pub struct SquareChannel {
    playing: bool,
    repeat: bool,
    frequency: u32,

    envelope_add_mode: bool,
    envelope_start_volume: u8,
    envelope_sweep_number: u8,

    sweep_inverse: bool,
    sweep_period: u8,
    sweep_shift: u8,

    wave_pattern_duty: u8,
    wave_length_load: u8,
}

//         Wave
// NR30 FF1A E--- ---- DAC power
// NR31 FF1B LLLL LLLL Length load (256-L)
// NR32 FF1C -VV- ---- Volume code (00=0%, 01=100%, 10=50%, 11=25%)
// NR33 FF1D FFFF FFFF Frequency LSB
// NR34 FF1E TL-- -FFF Trigger, Length enable, Frequency MSB
#[allow(dead_code)]
#[derive(Default)]
pub struct WaveChannel {
    playing: bool,
    repeat: bool,
    frequency: u32,

    wave_length_load: u8,
    wave_volume: u8,
}

//         Noise
// FF1F ---- ---- Not used
// NR41 FF20 --LL LLLL Length load (64-L)
// NR42 FF21 VVVV APPP Starting volume, Envelope add mode, period
// NR43 FF22 SSSS WDDD Clock shift, Width mode of LFSR, Divisor code
// NR44 FF23 TL-- ---- Trigger, Length enable
#[allow(dead_code)]
#[derive(Default)]
pub struct NoiseChannel {
    playing: bool,
    repeat: bool,

    envelope_add_mode: bool,
    envelope_start_volume: u8,
    envelope_sweep_number: u8,

    clock_shift: u8,
    clock_width_mode: u8,
    clock_divisor_code: u8,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct Sounder {
    // SO2
    left_speaker_volume: u8,
    left_speaker_master_enable: bool,
    left_speaker_channel1_enable: bool,
    left_speaker_channel2_enable: bool,
    left_speaker_channel3_enable: bool,
    left_speaker_channel4_enable: bool,

    // SO1
    right_speaker_volume: u8,
    right_speaker_master_enable: bool,
    right_speaker_channel1_enable: bool,
    right_speaker_channel2_enable: bool,
    right_speaker_channel3_enable: bool,
    right_speaker_channel4_enable: bool,

    // TONE & SWEEP
    channel1: SquareChannel,

    // TONE
    channel2: SquareChannel,

    // WAVE
    channel3: WaveChannel,

    // NOISE
    channel4: NoiseChannel,
}

impl Sounder {
    pub fn channel1_r0(&self) -> u8 {
        0
    }

    pub fn set_channel1_r0(&mut self, data: u8) {
        let _r = Channel1SweepControl::from_bits(data);
        println!("NR10 {:?}", _r);
    }

    pub fn channel1_r1(&self) -> u8 {
        0
    }

    pub fn set_channel1_r1(&mut self, data: u8) {
        let _r = Channel1SequenceControl::from_bits(data);
        println!("NR11 {:?}", _r);
    }

    pub fn channel1_r2(&self) -> u8 {
        0
    }

    pub fn set_channel1_r2(&mut self, data: u8) {
        let _r = Channel1EnvelopeControl::from_bits(data);
        println!("NR12 {:?}", _r);
    }

    pub fn channel1_r3(&self) -> u8 {
        0
    }

    pub fn set_channel1_r3(&mut self, data: u8) {
        let _r = Channel1FrequencyLowerData::from_bits(data);
        println!("NR13 {:?}", _r);
    }

    pub fn channel1_r4(&self) -> u8 {
        0
    }

    pub fn set_channel1_r4(&mut self, data: u8) {
        let _r = Channel1FrequencyHigherData::from_bits(data);
        println!("NR14 {:?}", _r);
    }

    pub fn channel2_r1(&self) -> u8 {
        0
    }

    pub fn set_channel2_r1(&mut self, data: u8) {
        let _r = Channel2SequenceControl::from_bits(data);
        println!("NR21 {:?}", _r);
    }

    pub fn channel2_r2(&self) -> u8 {
        0
    }

    pub fn set_channel2_r2(&mut self, data: u8) {
        let _r = Channel2EnvelopeControl::from_bits(data);
        println!("NR22 {:?}", _r);
    }

    pub fn channel2_r3(&self) -> u8 {
        0
    }

    pub fn set_channel2_r3(&mut self, data: u8) {
        let _r = Channel2FrequencyLowerData::from_bits(data);
        println!("NR23 {:?}", _r);
    }

    pub fn channel2_r4(&self) -> u8 {
        0
    }

    pub fn set_channel2_r4(&mut self, data: u8) {
        let _r = Channel2FrequencyHigherData::from_bits(data);
        println!("NR24 {:?}", _r);
    }

    pub fn channel3_r0(&self) -> u8 {
        0
    }

    pub fn set_channel3_r0(&mut self, data: u8) {
        let _r = Channel3SoundOnOffStatus::from_bits(data);
        println!("NR30 {:?}", _r);
    }

    pub fn channel3_r1(&self) -> u8 {
        0
    }

    pub fn set_channel3_r1(&mut self, data: u8) {
        let _r = Channel3SoundSequenceLength::from_bits(data);
        println!("NR31 {:?}", _r);
    }

    pub fn channel3_r2(&self) -> u8 {
        0
    }

    pub fn set_channel3_r2(&mut self, data: u8) {
        let _r = Channel3VolumeSelection::from_bits(data);
        println!("NR32 {:?}", _r);
    }

    pub fn channel3_r3(&self) -> u8 {
        0
    }

    pub fn set_channel3_r3(&mut self, data: u8) {
        let _r = Channel3FrequencyLowerData::from_bits(data);
        println!("NR33 {:?}", _r);
    }

    pub fn channel3_r4(&self) -> u8 {
        0
    }

    pub fn set_channel3_r4(&mut self, data: u8) {
        let _r = Channel3FrequencyHigherData::from_bits(data);
        println!("NR34 {:?}", _r);
    }

    pub fn channel3_sample(&self, index: u8) -> u8 {
        0
    }

    pub fn set_channel3_sample(&mut self, index: u8, data: u8) {
        //
    }

    pub fn channel4_r1(&self) -> u8 {
        0
    }

    pub fn set_channel4_r1(&mut self, data: u8) {
        let _r = Channel4SoundSequenceLength::from_bits(data);
        println!("NR41 {:?}", _r);
    }

    pub fn channel4_r2(&self) -> u8 {
        0
    }

    pub fn set_channel4_r2(&mut self, data: u8) {
        let _r = Channel4EnvelopeControl::from_bits(data);
        println!("NR42 {:?}", _r);
    }

    pub fn channel4_r3(&self) -> u8 {
        0
    }

    pub fn set_channel4_r3(&mut self, data: u8) {
        let _r = Channel4PolynomialCounterParameterControl::from_bits(data);
        println!("NR43 {:?}", _r);
    }

    pub fn channel4_r4(&self) -> u8 {
        0
    }

    pub fn set_channel4_r4(&mut self, data: u8) {
        let _r = Channel4PolynomialCounterSequenceControl::from_bits(data);
        println!("NR44 {:?}", _r);
    }

    pub fn master_r0(&self) -> u8 {
        0
    }

    pub fn set_master_r0(&mut self, data: u8) {
        let _r = MasterVolumeControl::from_bits(data);
        println!("NR50 {:?}", _r);
    }

    pub fn master_r1(&self) -> u8 {
        0
    }

    pub fn set_master_r1(&mut self, data: u8) {
        let _r = MasterOutputControl::from_bits(data);
        println!("NR51 {:?}", _r);
    }

    pub fn master_r2(&self) -> u8 {
        0
    }

    pub fn set_master_r2(&mut self, data: u8) {
        let _r = MasterOnOffControl::from_bits(data);
        println!("NR52 {:?}", _r);
    }
}