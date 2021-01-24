pub mod flags;

use flags::*;

use sdl2::audio::AudioQueue;

fn set_low_frequency_param(fparam: u32, low: u32) -> u32 {
    (fparam & 0x700) | (low & 0x0FF)
}

fn set_high_frequency_param(fparam: u32, high: u32) -> u32 {
    ((high & 0x7) << 8) | fparam & 0xFF
}

fn calculate_frequency(f: u32) -> u32 {
    131_072 / (2048 - f)
}

fn calculate_phase_duty(d: u8) -> f32 {
    match d {
        0 => 0.125,
        1 => 0.25,
        2 => 0.5,
        3 => 0.75,
        _ => 0.5
    }
}

fn calculate_volume(v: u8) -> i8 {
    let v: f32 = v as f32;
    let coef: f32 = 1.0 / 15.0;
    let maxv: f32 = 127.0;
    (v * coef * maxv) as i8
}

fn calculate_sample(s: u8) -> i8 {
    /*
    F |               **
    E |              *  *
    D |             *    *
    C |            *      *
    B |           *        *
    A |          *          *
    9 |         *            *
    8 |        *              *
    7 |       *                *
    6 |      *                  *
    5 |     *                    *
    4 |    *                      *
    3 |   *                        *
    2 |  *                          *
    1 | *                            *
    0 |*                              *
    *  --------------------------------
    */
    match s {
        0 => -127,
        1 => -109,
        2 => -90,
        3 => -72,
        4 => -54,
        5 => -36,
        6 => -18,
        7 => 0,
        8 => 0,
        9 => 18,
        10 => 36,
        11 => 54,
        12 => 72,
        13 => 90,
        14 => 109,
        15 => 127,
        _ => panic!(),
    }
}

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
pub struct SquareChannel {
    left_enable: bool,
    right_enable: bool,

    playing: bool,
    restart: bool,
    repeat: bool,

    frequency: u32,
    fparam: u32,

    envelope_direction: bool,
    envelope_start_volume: u8,
    envelope_sweep_number: u8,


    sweep_inverse: bool,
    sweep_period: u8,
    sweep_shift: u8,

    wave_duty: u8,
    wave_length: u8,

    buffer: Box<[i8; 8192]>,
    phase_duty: f32,
    phase_pos: f32,
    step_counter: f32,
    volume_step: u8,
    volume: i8,
}

impl Default for SquareChannel {
    fn default() -> Self {
        Self {
            left_enable: false,
            right_enable: false,

            playing: false,
            restart: false,
            repeat: false,
            frequency: calculate_frequency(0),
            fparam: 0,

            envelope_start_volume: 0,
            envelope_sweep_number: 0,
            envelope_direction: true,


            sweep_inverse: false,
            sweep_period: 0,
            sweep_shift: 0,
            wave_duty: 0,
            wave_length: 0,

            buffer: Box::new([0; 8192]),
            phase_duty: 0.5,
            phase_pos: 0.0,
            step_counter: 0.0,
            volume_step: 0,
            volume: 0,
        }
    }
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
    restart: bool,
    repeat: bool,
    frequency: u16,

    wave_length_load: u8,
    wave_volume: u8,
    wave_samples: [i8; 32],
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

pub trait SampleGenerator {
    fn enqueue_audio_samples(&mut self, queue: &mut AudioQueue<i8>);
}

impl SampleGenerator for SquareChannel {
    fn enqueue_audio_samples(&mut self, queue: &mut AudioQueue<i8>) {
        if self.restart {
            self.restart = false;
            self.playing = true;
            self.phase_pos = 0.0;
            queue.clear();
        }

        if !self.playing {
            return;
        }

        if !self.left_enable && !self.right_enable {
            return;
        }

        let phase_inc = self.frequency as f32 / queue.spec().freq as f32;
        let step_size = self.envelope_sweep_number as f32 * (queue.spec().freq as f32 / 64.0);

        let length = self.buffer.len();
        if (queue.size() as usize) < length {
            let length = length / 2;
            for i in 0..length {
                // envelope
                if step_size > 0.0 {
                    if self.step_counter >= step_size {
                        self.step_counter -= step_size;
                        if self.envelope_direction {
                            if self.volume_step < 0xF {
                                self.volume_step += 1;
                            }
                        } else {
                            if self.volume_step >= 0x1 {
                                self.volume_step -= 1;
                            }
                        }
                        self.volume = calculate_volume(self.volume_step);
                        if self.volume_step == 0 {
                            self.playing = false;
                        }
                    }
                    self.step_counter += 1.0;
                }

                let sample = self.volume * if self.phase_pos < self.phase_duty { 1 } else { -1 };

                self.phase_pos += phase_inc;
                self.phase_pos %= 1.0;

                // left
                self.buffer[i * 2] = if self.left_enable { sample } else { 0 };

                // right
                self.buffer[i * 2 + 1] = if self.right_enable { sample } else { 0 };
            }
            queue.queue(&*self.buffer);
        }
    }
}

impl SampleGenerator for NoiseChannel {
    fn enqueue_audio_samples(&mut self, _queue: &mut AudioQueue<i8>) {

    }
}

impl SampleGenerator for WaveChannel {
    fn enqueue_audio_samples(&mut self, _queue: &mut AudioQueue<i8>) {

    }
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

        self.channel1.phase_duty = calculate_phase_duty(self.channel1.wave_duty);

        println!("NR11 ch1_duty={} ch1_len={} ch1_duty={}",
            self.channel1.wave_duty,
            self.channel1.wave_length,
            self.channel1.phase_duty);
    }

    pub fn channel1_r2(&self) -> u8 {
        0
    }

    pub fn set_channel1_r2(&mut self, data: u8) {
        let r = Channel1EnvelopeControl::from_bits(data).unwrap();
        self.channel1.envelope_start_volume = (r & Channel1EnvelopeControl::ENVELOPE_INITIAL_VOLUME_MASK).bits() >> 4;
        self.channel1.envelope_direction = r.contains(Channel1EnvelopeControl::ENVELOPE_DIRECTION_SELECT);
        self.channel1.envelope_sweep_number = (r & Channel1EnvelopeControl::ENVELOPE_SWEEP_NUMBER_MASK).bits();
        self.channel1.volume_step = self.channel1.envelope_start_volume;
        self.channel1.volume = calculate_volume(self.channel1.volume_step);

        println!("NR12 ch1_env_start_vol={} ch1_env_dir={} ch1_env_num={} ch1_vol={}",
            self.channel1.envelope_start_volume,
            self.channel1.envelope_direction,
            self.channel1.envelope_sweep_number,
            self.channel1.volume);
    }

    pub fn channel1_r3(&self) -> u8 {
        0
    }

    pub fn set_channel1_r3(&mut self, data: u8) {
        self.channel1.fparam = set_low_frequency_param(self.channel1.fparam, data as u32);
        self.channel1.frequency = calculate_frequency(self.channel1.fparam);
        println!("NR13 ch1_fparam={} ch1_freq={}", self.channel1.fparam, self.channel1.frequency);
    }

    pub fn channel1_r4(&self) -> u8 {
        0
    }

    pub fn set_channel1_r4(&mut self, data: u8) {
        let r = Channel1FrequencyHigherData::from_bits(data).unwrap();
        self.channel1.fparam = set_high_frequency_param(self.channel1.fparam, data as u32);
        self.channel1.frequency = calculate_frequency(self.channel1.fparam);
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
        let r = Channel2SequenceControl::from_bits(data).unwrap();

        self.channel2.wave_duty = (r & Channel2SequenceControl::SOUND_SEQUENCE_DUTY_MASK).bits() >> 6;
        self.channel2.wave_length = (r & Channel2SequenceControl::SOUND_SEQUENCE_LENGTH_MASK).bits();
        self.channel2.phase_duty = calculate_phase_duty(self.channel2.wave_duty);

        println!("NR21 ch2_duty={} ch2_len={} ch2_duty={}",
            self.channel2.wave_duty,
            self.channel2.wave_length,
            self.channel2.phase_duty);
    }

    pub fn channel2_r2(&self) -> u8 {
        0
    }

    pub fn set_channel2_r2(&mut self, data: u8) {
        let r = Channel2EnvelopeControl::from_bits(data).unwrap();

        self.channel2.envelope_start_volume = (r & Channel2EnvelopeControl::ENVELOPE_INITIAL_VOLUME_MASK).bits() >> 4;
        self.channel2.envelope_direction = r.contains(Channel2EnvelopeControl::ENVELOPE_DIRECTION_SELECT);
        self.channel2.envelope_sweep_number = (r & Channel2EnvelopeControl::ENVELOPE_SWEEP_NUMBER_MASK).bits();
        self.channel2.volume = calculate_volume(self.channel2.envelope_start_volume);

        println!("NR22 ch2_env_start_vol={} ch2_env_dir={} ch2_env_num={} ch2_vol={}",
            self.channel1.envelope_start_volume,
            self.channel1.envelope_direction,
            self.channel1.envelope_sweep_number,
            self.channel1.volume);
    }

    pub fn channel2_r3(&self) -> u8 {
        0
    }

    pub fn set_channel2_r3(&mut self, data: u8) {
        self.channel2.fparam = set_low_frequency_param(self.channel2.fparam, data as u32);
        self.channel2.frequency = calculate_frequency(self.channel2.fparam);

        println!("NR23 ch2_fparam={} ch2_freq={}", self.channel2.fparam, self.channel2.frequency);
    }

    pub fn channel2_r4(&self) -> u8 {
        0
    }

    pub fn set_channel2_r4(&mut self, data: u8) {
        let r = Channel2FrequencyHigherData::from_bits(data).unwrap();

        self.channel2.fparam = set_high_frequency_param(self.channel2.fparam, data as u32);
        self.channel2.repeat = !r.contains(Channel2FrequencyHigherData::STOP_ON_SEQUENCE_COMPLETE);
        self.channel2.restart = r.contains(Channel2FrequencyHigherData::RESTART_SEQUENCE);

        println!("NR24 ch2_fparam={} ch2_freq={} ch2_repeat={} ch2_restart={}",
            self.channel2.fparam,
            self.channel2.frequency,
            self.channel2.repeat,
            self.channel2.restart);
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

    pub fn channel3_sample(&self, _index: u8) -> u8 {
        0
    }

    pub fn set_channel3_sample(&mut self, index: u8, data: u8) {
        let index = index as usize;
        let sample1 = calculate_sample((data & 0xF0) >> 4);
        let sample2 = calculate_sample(data &0x0F);
        println!("CH3 S[{}]={} S[{}]={}", 2 * index, sample1, 2 * index + 1, sample2);

        self.channel3.wave_samples[2 * index] = sample1;
        self.channel3.wave_samples[2 * index + 1] = sample2;
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

    pub fn enqueue_audio_samples(&mut self, channels: &mut [AudioQueue<i8>; 4]) {
        self.channel1.enqueue_audio_samples(&mut channels[0]);
        self.channel2.enqueue_audio_samples(&mut channels[1]);
        self.channel3.enqueue_audio_samples(&mut channels[2]);
        self.channel4.enqueue_audio_samples(&mut channels[3]);
    }
}