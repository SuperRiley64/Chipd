use rand::random;
use std::fs;

const MEMORY_SIZE: usize = 4096;
const DISPLAY_WIDTH: usize = 64;
const DISPLAY_HEIGHT: usize = 32;
const PROGRAM_START: u16 = 0x200;

pub struct Cpu {
    pub memory: [u8; MEMORY_SIZE],
    pub v: [u8; 16],
    pub i: u16,
    pub pc: u16,

    pub stack: [u16; 16],
    pub sp: usize,

    pub delay_timer: u8,
    pub sound_timer: u8,

    pub display: [bool; DISPLAY_WIDTH * DISPLAY_HEIGHT],
    pub keys: [bool; 16],

    pub screen_update: bool,
    pub paused: bool,
    pub blocked: bool,
    pub key_register: Option<usize>,
}

impl Cpu {
    pub fn new() -> Self {
        let mut cpu = Self {
            memory: [0; MEMORY_SIZE],
            v: [0; 16],
            i: 0,
            pc: PROGRAM_START,

            stack: [0; 16],
            sp: 0,

            delay_timer: 0,
            sound_timer: 0,

            display: [false; DISPLAY_WIDTH * DISPLAY_HEIGHT],
            keys: [false; 16],

            screen_update: false,
            paused: false,
            blocked: false,
            key_register: None,
        };

        cpu.load_fontset();
        cpu
    }

    fn load_fontset(&mut self) {
        let fontset: [u8; 80] = [
            0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
            0x20, 0x60, 0x20, 0x20, 0x70, // 1
            0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
            0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
            0x90, 0x90, 0xF0, 0x10, 0x10, // 4
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
            0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
            0xF0, 0x10, 0x20, 0x40, 0x40, // 7
            0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
            0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
            0xF0, 0x90, 0xF0, 0x90, 0x90, // A
            0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
            0xF0, 0x80, 0x80, 0x80, 0xF0, // C
            0xE0, 0x90, 0x90, 0x90, 0xE0, // D
            0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
            0xF0, 0x80, 0xF0, 0x80, 0x80, // F
        ];

        self.memory[0x000..0x050].copy_from_slice(&fontset);
    }

    pub fn load_rom(&mut self, filename: &str) -> std::io::Result<()> {
        let rom = fs::read(filename)?;

        for (offset, byte) in rom.iter().enumerate() {
            self.memory[PROGRAM_START as usize + offset] = *byte;
        }

        Ok(())
    }

    fn fetch_opcode(&self) -> u16 {
        let high = self.memory[self.pc as usize] as u16;
        let low = self.memory[(self.pc + 1) as usize] as u16;

        (high << 8) | low
    }

    pub fn emu_cycle(&mut self) {
        if self.blocked {
            return;
        }

        let opcode = self.fetch_opcode();

        let x = ((opcode & 0x0F00) >> 8) as usize;
        let y = ((opcode & 0x00F0) >> 4) as usize;
        let n = (opcode & 0x000F) as u8;
        let nn = (opcode & 0x00FF) as u8;
        let nnn = opcode & 0x0FFF;

        match opcode & 0xF000 {
            0x1000 => self.pc = nnn,

            0x2000 => {
                self.stack[self.sp] = self.pc + 2;
                self.sp += 1;
                self.pc = nnn;
            }

            0x3000 => {
                self.pc += if self.v[x] == nn { 4 } else { 2 };
            }

            0x4000 => {
                self.pc += if self.v[x] != nn { 4 } else { 2 };
            }

            0x6000 => {
                self.v[x] = nn;
                self.pc += 2;
            }

            0x7000 => {
                self.v[x] = self.v[x].wrapping_add(nn);
                self.pc += 2;
            }

            0xA000 => {
                self.i = nnn;
                self.pc += 2;
            }

            0xB000 => {
                self.pc = nnn + self.v[0] as u16;
            }

            0xC000 => {
                self.v[x] = random::<u8>() & nn;
                self.pc += 2;
            }

            0xD000 => {
                self.draw_sprite(x, y, n);
                self.pc += 2;
            }

            _ => self.decode_extended(opcode, x, y, nn),
        }
    }

    fn decode_extended(&mut self, opcode: u16, x: usize, y: usize, nn: u8) {
        match opcode {
            0x00E0 => {
                self.display = [false; DISPLAY_WIDTH * DISPLAY_HEIGHT];
                self.screen_update = true;
                self.pc += 2;
            }

            0x00EE => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            }

            _ => match opcode & 0xF00F {
                0x5000 => {
                    self.pc += if self.v[x] == self.v[y] { 4 } else { 2 };
                }

                0x8000 => {
                    self.v[x] = self.v[y];
                    self.pc += 2;
                }

                0x8001 => {
                    self.v[x] |= self.v[y];
                    self.pc += 2;
                }

                0x8002 => {
                    self.v[x] &= self.v[y];
                    self.pc += 2;
                }

                0x8003 => {
                    self.v[x] ^= self.v[y];
                    self.pc += 2;
                }

                0x8004 => {
                    let (result, carry) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = if carry { 1 } else { 0 };
                    self.pc += 2;
                }

                0x8005 => {
                    let (result, borrow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = result;
                    self.v[0xF] = if borrow { 0 } else { 1 };
                    self.pc += 2;
                }

                0x8006 => {
                    self.v[0xF] = self.v[x] & 0x01;
                    self.v[x] >>= 1;
                    self.pc += 2;
                }

                0x8007 => {
                    let (result, borrow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = result;
                    self.v[0xF] = if borrow { 0 } else { 1 };
                    self.pc += 2;
                }

                0x800E => {
                    self.v[0xF] = (self.v[x] & 0x80) >> 7;
                    self.v[x] <<= 1;
                    self.pc += 2;
                }

                0x9000 => {
                    self.pc += if self.v[x] != self.v[y] { 4 } else { 2 };
                }

                _ => self.decode_f_family(opcode, x, nn),
            },
        }
    }

    fn decode_f_family(&mut self, opcode: u16, x: usize, _nn: u8) {
        match opcode & 0xF0FF {
            0xE09E => {
                let key = self.v[x] as usize;
                self.pc += if self.keys[key] { 4 } else { 2 };
            }

            0xE0A1 => {
                let key = self.v[x] as usize;
                self.pc += if !self.keys[key] { 4 } else { 2 };
            }

            0xF007 => {
                self.v[x] = self.delay_timer;
                self.pc += 2;
            }

            0xF00A => {
                self.blocked = true;
                self.key_register = Some(x);
            }

            0xF015 => {
                self.delay_timer = self.v[x];
                self.pc += 2;
            }

            0xF018 => {
                self.sound_timer = self.v[x];
                self.pc += 2;
            }

            0xF01E => {
                self.i = self.i.wrapping_add(self.v[x] as u16);
                self.pc += 2;
            }

            0xF029 => {
                self.i = self.v[x] as u16 * 5;
                self.pc += 2;
            }

            0xF033 => {
                let value = self.v[x];
                let i = self.i as usize;

                self.memory[i] = value / 100;
                self.memory[i + 1] = (value / 10) % 10;
                self.memory[i + 2] = value % 10;

                self.pc += 2;
            }

            0xF055 => {
                let i = self.i as usize;

                for offset in 0..=x {
                    self.memory[i + offset] = self.v[offset];
                }

                self.pc += 2;
            }

            0xF065 => {
                let i = self.i as usize;

                for offset in 0..=x {
                    self.v[offset] = self.memory[i + offset];
                }

                self.pc += 2;
            }

            _ => {
                println!("Opcode not found: {:#06X}", opcode);
                self.pc += 2;
            }
        }
    }

    fn draw_sprite(&mut self, x_reg: usize, y_reg: usize, height: u8) {
        self.screen_update = true;

        let x_pos = self.v[x_reg] as usize;
        let y_pos = self.v[y_reg] as usize;

        self.v[0xF] = 0;

        for row in 0..height as usize {
            let sprite_byte = self.memory[self.i as usize + row];

            for bit in 0..8 {
                let pixel_on = (sprite_byte & (0x80 >> bit)) != 0;

                if pixel_on {
                    let x = (x_pos + bit) % DISPLAY_WIDTH;
                    let y = (y_pos + row) % DISPLAY_HEIGHT;
                    let index = y * DISPLAY_WIDTH + x;

                    if self.display[index] {
                        self.v[0xF] = 1;
                    }

                    self.display[index] = !self.display[index];
                }
            }
        }
    }

    pub fn update_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            // Sound system can handle this later.
            self.sound_timer -= 1;
        }
    }

    pub fn press_key(&mut self, index: usize) {
        self.keys[index] = true;

        if self.blocked {
            if let Some(register) = self.key_register {
                self.v[register] = index as u8;
            }

            self.key_register = None;
            self.blocked = false;
            self.pc += 2;
        }
    }

    pub fn release_key(&mut self, index: usize) {
        self.keys[index] = false;
    }

    pub fn reset(&mut self) {
        self.memory[PROGRAM_START as usize..].fill(0);
        self.v.fill(0);
        self.stack.fill(0);
        self.keys.fill(false);
        self.display.fill(false);

        self.i = 0;
        self.pc = PROGRAM_START;
        self.sp = 0;

        self.delay_timer = 0;
        self.sound_timer = 0;

        self.screen_update = true;
        self.paused = false;
        self.blocked = false;
        self.key_register = None;

        self.load_fontset();
    }
}