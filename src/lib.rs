use anyhow::{Context as _, Result};

pub const ADD_A: u8 = 40;
pub const DEC_A: u8 = 64;
pub const HALT: u8 = 0;
pub const INC_A: u8 = 48;
pub const JMP: u8 = 247;
pub const LD_A: u8 = 16;

#[expect(clippy::min_ident_chars, reason = "some regs have single-letter names")]
#[derive(Debug)]
pub struct Cpu {
    pub a: u8,
    pub mem: [u8; 65536],
    pub pc: u16,
    pub zero: bool,
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            a: 0,
            mem: [0; 65536],
            pc: 0,
            zero: false,
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

    /// Loads `program` into memory.
    ///
    /// # Errors
    ///
    /// * If the program does not fit in memory.
    pub fn load_program(&mut self, program: &[u8]) -> Result<()> {
        let mem = self
            .mem
            .get_mut(0..program.len())
            .context("program too long")?;
        mem.copy_from_slice(program);
        Ok(())
    }

    pub fn run(&mut self) {
        while self.step() {}
    }

    /// Runs `program`.
    ///
    /// # Errors
    ///
    /// * If the program does not fit in memory.
    pub fn run_program(&mut self, program: &[u8]) -> Result<()> {
        self.load_program(program)?;
        self.run();
        Ok(())
    }

    pub fn step(&mut self) -> bool {
        let opcode = self.fetch();
        match opcode {
            ADD_A => {
                let operand = self.fetch();
                self.a = self.a.wrapping_add(operand);
                self.update_zero(self.a);
            }
            DEC_A => {
                self.a = self.a.wrapping_sub(1);
                self.update_zero(self.a);
            }
            HALT => return false,
            INC_A => {
                self.a = self.a.wrapping_add(1);
                self.update_zero(self.a);
            }
            JMP => {
                let addr_lo = self.fetch();
                let addr_hi = self.fetch();
                let addr = u16::from_le_bytes([addr_lo, addr_hi]);
                self.pc = addr;
            }
            LD_A => {
                let operand = self.fetch();
                self.a = operand;
                self.update_zero(self.a);
            }
            _ => {}
        }
        true
    }

    pub fn update_zero(&mut self, value: u8) {
        self.zero = value == 0;
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
        assert_eq!(cpu.zero, false, "wrong initial zero flag");
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
        cpu.zero = true;
        cpu.run_program(&[INC_A, HALT]).unwrap();
        assert_eq!(cpu.a, 1, "wrong A after `inc a`");
        assert_eq!(cpu.zero, false, "zero not cleared");
    }

    #[test]
    fn a_goes_from_255_to_0() {
        let mut cpu = Cpu::default();
        cpu.a = 255;
        cpu.run_program(&[INC_A, HALT]).unwrap();
        assert_eq!(cpu.a, 0, "wrong A after `inc a` past 255");
        assert_eq!(cpu.zero, true, "zero not set");
    }

    #[test]
    fn a_decrements_from_0_to_255() {
        let mut cpu = Cpu::default();
        cpu.a = 0;
        cpu.zero = true;
        cpu.run_program(&[DEC_A, HALT]).unwrap();
        assert_eq!(cpu.a, 255, "wrong A after `dec a` below 0");
        assert_eq!(cpu.zero, false, "zero not cleared");
    }

    #[test]
    fn dec_sets_zero_flag() {
        let mut cpu = Cpu::default();
        cpu.a = 1;
        cpu.run_program(&[DEC_A, HALT]).unwrap();
        assert_eq!(cpu.zero, true, "zero not set");
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
        cpu.run_program(&[INC_A, HALT]).unwrap();
        assert_eq!(cpu.a, 1, "wrong A after `inc a`");
        assert_eq!(cpu.pc, 2, "wrong PC after run()");
    }

    #[test]
    fn ld_loads_accumulator() {
        let mut cpu = Cpu::default();
        cpu.a = 10;
        cpu.zero = true;
        cpu.run_program(&[LD_A, 5, HALT]).unwrap();
        assert_eq!(cpu.a, 5, "wrong A after `ld a`");
        assert_eq!(cpu.pc, 3, "wrong PC after run()");
        assert_eq!(cpu.zero, false, "zero not cleared");
    }

    #[test]
    fn ld_sets_zero_flag() {
        let mut cpu = Cpu::default();
        cpu.a = 10;
        cpu.run_program(&[LD_A, 0, HALT]).unwrap();
        assert_eq!(cpu.zero, true, "zero not set");
    }

    #[test]
    fn add_adds_to_accumulator() {
        let mut cpu = Cpu::default();
        cpu.a = 1;
        cpu.zero = true;
        cpu.run_program(&[ADD_A, 2, HALT]).unwrap();
        assert_eq!(cpu.a, 3, "wrong A after `add a`");
        assert_eq!(cpu.pc, 3, "wrong PC after run()");
        assert_eq!(cpu.zero, false, "zero not cleared");
    }

    #[test]
    fn add_sets_zero_flag() {
        let mut cpu = Cpu::default();
        cpu.a = 1;
        cpu.run_program(&[ADD_A, 255, HALT]).unwrap();
        assert_eq!(cpu.zero, true, "zero not set");
    }

    #[test]
    fn jmp_jumps_to_specified_address() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[JMP, 5, 0, INC_A, HALT, HALT]).unwrap();
        assert_eq!(cpu.a, 0, "wrong A");
        assert_eq!(cpu.pc, 6, "wrong PC after run()");
    }
}
