pub const DEC_A: u8 = 64;
pub const HALT: u8 = 0;
pub const INC_A: u8 = 48;
pub const LD_A: u8 = 16;

#[expect(clippy::min_ident_chars, reason = "some regs have single-letter names")]
#[derive(Debug)]
pub struct Cpu {
    pub a: u8,
    pub mem: [u8; 65536],
    pub pc: u16,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            mem: [0; 65536],
            pc: 0,
        }
    }
}

impl Cpu {
    pub fn fetch(&mut self) -> u8 {
        let address = usize::from(self.pc);
        let value = self.mem.get(address).copied().unwrap_or_default();
        self.pc = self.pc.wrapping_add(1);
        value
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    pub fn step(&mut self) -> bool {
        let opcode = self.fetch();
        match opcode {
            DEC_A => self.a = self.a.wrapping_sub(1),
            HALT => return false,
            INC_A => self.a = self.a.wrapping_add(1),
            LD_A => {
                let operand = self.fetch();
                self.a = operand;
            }
            _ => {}
        }
        true
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests")]
mod tests {
    use super::*;

    #[test]
    fn default_correctly_initialises_cpu() {
        let cpu = Cpu::default();
        assert_eq!(cpu.a, 0, "wrong initial A");
        assert_eq!(cpu.pc, 0, "wrong initial PC");
        assert_eq!(*cpu.mem.first().unwrap(), 0, "wrong memory contents");
    }

    #[test]
    fn step_increments_pc() {
        let mut cpu = Cpu::default();
        cpu.step();
        assert_eq!(cpu.pc, 1, "wrong PC after step()");
        cpu.step();
        assert_eq!(cpu.pc, 2, "wrong PC after step()");
    }

    #[test]
    fn inc_increments_a() {
        let mut cpu = Cpu::default();
        cpu.mem[0] = INC_A;
        cpu.step();
        assert_eq!(cpu.a, 1, "wrong A after `inc a`");
    }

    #[test]
    fn a_goes_from_255_to_0() {
        let mut cpu = Cpu::default();
        cpu.a = 255;
        cpu.mem[0] = INC_A;
        cpu.step();
        assert_eq!(cpu.a, 0, "wrong A after `inc a` past 255");
    }

    #[test]
    fn a_decrements_from_0_to_255() {
        let mut cpu = Cpu::default();
        cpu.a = 0;
        cpu.mem[0] = DEC_A;
        cpu.step();
        assert_eq!(cpu.a, 255, "wrong A after `dec a` below 0");
    }

    #[test]
    fn pc_goes_from_65535_to_0() {
        let mut cpu = Cpu::default();
        cpu.pc = 65535;
        cpu.step();
        assert_eq!(cpu.pc, 0, "wrong PC after 65535");
    }

    #[test]
    fn memory_is_bytes() {
        let mut cpu = Cpu::default();
        cpu.mem[0] = 0_u8;
        assert_eq!(cpu.mem[0], 0, "wrong memory contents");
    }

    #[test]
    fn run_runs_until_halted() {
        let mut cpu = Cpu::default();
        cpu.mem[0] = INC_A;
        cpu.mem[1] = HALT;
        cpu.run();
        assert_eq!(cpu.a, 1, "wrong A after `inc a`");
        assert_eq!(cpu.pc, 2, "wrong PC after run()");
    }

    #[test]
    fn ld_loads_accumulator() {
        let mut cpu = Cpu::default();
        cpu.a = 10;
        cpu.mem[0] = LD_A;
        cpu.mem[1] = 5;
        cpu.step();
        assert_eq!(cpu.a, 5, "wrong A after `ld a`");
        assert_eq!(cpu.pc, 2, "wrong PC after step()");
    }
}
