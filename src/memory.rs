pub struct MMU {
    memory: [u8; 1024],
}

impl MMU {
    // constructor
    pub fn new() -> MMU {
        return MMU { memory: [0; 1024] };
    }

    // accessors
    pub fn read_byte(&self, address: usize) -> u8 {
        return self.memory[address];
    }

    pub fn write_byte(&mut self, address: usize, value: u8) {
        self.memory[address] = value;
    }

    pub fn read_word(&self, address: usize) -> u16 {
        u16::from_le_bytes(self.memory[address..(address + 2)].try_into().unwrap())
    }

    pub fn write_word(&mut self, address: usize, value: u16) {
        let value_bytes = value.to_le_bytes();
        self.memory[address] = value_bytes[0];
        self.memory[address + 1] = value_bytes[1];
    }
}
