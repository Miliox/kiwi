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
    left_enable: bool,
    right_enable: bool,

    playing: bool,
    restart: bool,
    repeat: bool,

    frequency: u32,
    fparam: u32,

    envelope_add_mode: bool,
    envelope_start_volume: u8,
    envelope_sweep_number: u8,

    sweep_inverse: bool,
    sweep_period: u8,
    sweep_shift: u8,

    wave_duty: u8,
    wave_length: u8,
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
    left_enable: bool,
    right_enable: bool,

    playing: bool,
    repeat: bool,
    frequency: u16,

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
    left_enable: bool,
    right_enable: bool,

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
    enable: bool,

    // SO2
    left_volume: u8,

    // SO1
    right_volume: u8,

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
        let r = Channel1SweepControl::from_bits(data).unwrap();
        self.channel1.sweep_inverse = r.contains(Channel1SweepControl::SWEEP_DIRECTION_SELECT);
        self.channel1.sweep_period = (r & Channel1SweepControl::SWEEP_PERIOD_MASK).bits() >> 4;
        self.channel1.sweep_shift = (r & Channel1SweepControl::SWEEP_SHIFT_MASK).bits();
        println!("NR10 ch1_sweep_inv={} ch1_sweep_period={} ch1_sweep_shift={}",
            self.channel1.sweep_inverse,
            self.channel1.sweep_period,
            self.channel1.sweep_shift);
    }

    pub fn channel1_r1(&self) -> u8 {
        0
    }

    pub fn set_channel1_r1(&mut self, data: u8) {
        let r = Channel1SequenceControl::from_bits(data).unwrap();
        self.channel1.wave_duty = (r & Channel1SequenceControl::SOUND_SEQUENCE_DUTY_MASK).bits() >> 6;
        self.channel1.wave_length = (r & Channel1SequenceControl::SOUND_SEQUENCE_LENGTH_MASK).bits();
        println!("NR11 ch1_duty={} ch1_len={}", self.channel1.wave_duty, self.channel1.wave_length);
    }

    pub fn channel1_r2(&self) -> u8 {
        0
    }

    pub fn set_channel1_r2(&mut self, data: u8) {
        let r = Channel1EnvelopeControl::from_bits(data).unwrap();
        self.channel1.envelope_start_volume = (r & Channel1EnvelopeControl::ENVELOPE_INITIAL_VOLUME_MASK).bits() >> 4;
        self.channel1.envelope_add_mode = r.contains(Channel1EnvelopeControl::ENVELOPE_DIRECTION_SELECT);
        self.channel1.envelope_sweep_number = (r & Channel1EnvelopeControl::ENVELOPE_SWEEP_NUMBER_MASK).bits();
        println!("NR12 ch1_env_start_vol={} ch1_env_add_mode={} ch1_env_num={}",
            self.channel1.envelope_start_volume,
            self.channel1.envelope_add_mode,
            self.channel1.envelope_sweep_number);
    }

    pub fn channel1_r3(&self) -> u8 {
        0
    }

    pub fn set_channel1_r3(&mut self, data: u8) {
        self.channel1.fparam = (self.channel1.fparam & 0x300) | data as u32;
        self.channel1.frequency = Self::calculate_frequency(self.channel1.fparam);
        println!("NR13 ch1_fparam={} ch1_freq={}", self.channel1.fparam, self.channel1.frequency);
    }

    pub fn channel1_r4(&self) -> u8 {
        0
    }

    pub fn set_channel1_r4(&mut self, data: u8) {
        let r = Channel1FrequencyHigherData::from_bits(data).unwrap();
        self.channel1.fparam = (self.channel1.fparam & 0xFF) | (data as u32 & 0x3) << 8;
        self.channel1.repeat = !r.contains(Channel1FrequencyHigherData::STOP_ON_SEQUENCE_COMPLETE);
        self.channel1.restart = r.contains(Channel1FrequencyHigherData::RESTART_SEQUENCE);
        println!("NR14 ch1_fparam={} ch1_freq={} ch1_repeat={} ch1_restart={}",
            self.channel1.fparam,
            self.channel1.frequency,
            self.channel1.repeat,
            self.channel1.restart);
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
        let r = MasterVolumeControl::from_bits(data).unwrap();

        self.left_volume = (r & MasterVolumeControl::LEFT_CHANNEL_VOLUME_MASK).bits() >> 4;
        self.right_volume = (r & MasterVolumeControl::RIGHT_CHANNEL_VOLUME_MASK).bits() >> 0;

        println!("NR50 volume=({}, {})", self.left_volume, self.right_volume);
    }

    pub fn master_r1(&self) -> u8 {
        0
    }

    pub fn set_master_r1(&mut self, data: u8) {
        let r = MasterOutputControl::from_bits(data).unwrap();

        self.channel4.left_enable = r.contains(MasterOutputControl::LEFT_CHANNEL_4_ENABLE);
        self.channel3.left_enable = r.contains(MasterOutputControl::LEFT_CHANNEL_3_ENABLE);
        self.channel2.left_enable = r.contains(MasterOutputControl::LEFT_CHANNEL_2_ENABLE);
        self.channel1.left_enable = r.contains(MasterOutputControl::LEFT_CHANNEL_1_ENABLE);

        self.channel4.right_enable = r.contains(MasterOutputControl::RIGHT_CHANNEL_4_ENABLE);
        self.channel3.right_enable = r.contains(MasterOutputControl::RIGHT_CHANNEL_3_ENABLE);
        self.channel2.right_enable = r.contains(MasterOutputControl::RIGHT_CHANNEL_2_ENABLE);
        self.channel1.right_enable = r.contains(MasterOutputControl::RIGHT_CHANNEL_1_ENABLE);

        println!("NR51 ch1_on=({}, {}) ch2_on=({}, {}) ch3_on=({}, {}) ch4_on=({}, {}))",
            self.channel1.left_enable, self.channel1.right_enable,
            self.channel2.left_enable, self.channel2.right_enable,
            self.channel3.left_enable, self.channel3.right_enable,
            self.channel4.left_enable, self.channel4.right_enable);
    }

    pub fn master_r2(&self) -> u8 {
        0
    }

    pub fn set_master_r2(&mut self, data: u8) {
        let r = MasterOnOffControl::from_bits(data).unwrap();
        self.enable = r.contains(MasterOnOffControl::CHANNEL_ALL_ENABLE);
        println!("NR52 sound_on={}", self.enable);
    }

    fn calculate_frequency(f: u32) -> u32 {
        131_072 / (2048 - f)
    }
}