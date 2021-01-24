#[allow(dead_code)]
pub struct Cartridge {
    pub rom: Vec<u8>,
    pub ram: Box<[u8; 0x2000]>,

    switchable_rom_bank_offset: usize,
}

fn rom_bank_count(rom_size_code: u8) -> usize {
    match rom_size_code {
        0 => 2,
        1 => 4,
        2 => 8,
        3 => 16,
        4 => 32,
        5 => 64,
        6 => 128,
        _ => panic!(""),
    }
}

#[allow(dead_code)]
impl Cartridge {
    pub fn new() -> Self {
        Self {
            rom: Vec::new(),
            ram: Box::new([0; 0x2000]),
            switchable_rom_bank_offset: 0x4000,
        }
    }

    pub fn open(&mut self, filename: &str) {
        self.rom = std::fs::read(filename).unwrap();

        println!("Cartridge title={} type={} rom={} ram={}",
            self.title(),
            self.cart_type(),
            self.rom_size_code(),
            self.ram_size_code());
    }

    pub fn read_rom(&self, addr: u16) -> u8 {
        let mut addr = addr as usize;
        if addr >= 0x4000 {
            addr -= 0x4000;
            addr += self.switchable_rom_bank_offset;
        }
        self.rom[addr]
    }

    pub fn write_rom(&mut self, addr: u16, data: u8) {
        match self.cart_type() {
            1 => {
                match addr {
                    0x2000..=0x3FFF => {
                        self.switchable_rom_bank_offset = match data {
                            0 => 0x4000,
                            1..=6 => 0x4000 * data as usize,
                            _ => panic!("{}", data)
                        };
                        println!("Switch Cartridge Bank {} {:04X}", data, self.switchable_rom_bank_offset);
                    }
                    _ => { }
                }
            }
            _ => {}
        }
    }

    pub fn read_ram(&self, addr: u16) -> u8 {
        self.ram[addr as usize]
    }

    pub fn write_ram(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    pub fn title(&self) -> String {
        std::str::from_utf8(&self.rom[0x134..=0x142]).unwrap().to_string()
    }

    pub fn is_color(&self) -> bool {
        self.rom[0x0143] == 0x80
    }

    pub fn is_super(&self) -> bool {
        self.rom[0x0146] == 0x03
    }

    pub fn is_japanese(&self) -> bool {
        self.rom[0x014a] == 0x00
    }

    pub fn entry_point(&self) -> Vec<u8> {
        self.rom[0x100..=0x103].to_vec()
    }

    pub fn lincense_code(&self) -> u16 {
        let old_license = self.rom[0x144];
        if old_license != 0x33 {
            old_license as u16
        } else {
            u16::from_be_bytes([
                self.rom[0x144],
                self.rom[0x145]])
        }
    }

    pub fn cart_type(&self) -> u8 {
        self.rom[0x147]
    }

    pub fn rom_size_code(&self) -> u8 {
        self.rom[0x148]
    }

    pub fn ram_size_code(&self) -> u8 {
        self.rom[0x149]
    }

    pub fn rom_slice(&self, addr: u16, size: u16) -> Vec<u8> {
        self.rom[addr as usize..((addr + size) as usize)].to_vec()
    }

    pub fn complement_check(&self) -> u8 {
        self.rom[0x14d]
    }

    pub fn checksum(&self) -> u16 {
        u16::from_be_bytes([
            self.rom[0x14e],
            self.rom[0x14f]])
    }
}

impl Default for Cartridge {
    fn default() -> Self { Self::new() }
}