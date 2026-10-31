use anyhow::{Context as _, Result};

pub const ADD_A: u8 = 80;
pub const BEQ: u8 = 241;
pub const BRA: u8 = 240;
pub const CMP_A: u8 = 112;
pub const DEC_A: u8 = 64;
pub const HALT: u8 = 0;
pub const INC_A: u8 = 48;
pub const JMP: u8 = 247;
pub const LD_A: u8 = 16;
pub const TRAP: u8 = 249;

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
            BEQ => {
                let displacement = self.fetch();
                if self.zero {
                    self.pc = self.pc.wrapping_add(u16::from(displacement));
                }
            }
            BRA => {
                let displacement = self.fetch();
                self.pc = self.pc.wrapping_add(u16::from(displacement));
            }
            CMP_A => {
                let operand = self.fetch();
                self.zero = self.a == operand;
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
            TRAP => {
                let operand = self.fetch();
                if operand == 32 {
                    print!("{}", char::from(self.a));
                }
            }
            _ => {}
        }
        true
    }

    pub fn update_zero(&mut self, value: u8) {
        self.zero = value == 0;
    }
}

#[must_use]
pub fn disassemble(code: &[u8]) -> (String, usize) {
    let Some(&opcode) = code.first() else {
        return ("???".to_owned(), 0);
    };
    let op_lo = code.get(1);
    let op_hi = code.get(2);
    match opcode {
        BEQ => (format!("beq {}", format_byte(op_lo)), 2),
        CMP_A => (format!("cmp a, {}", format_byte(op_lo)), 2),
        DEC_A => ("dec a".to_owned(), 1),
        HALT => ("halt".to_owned(), 1),
        INC_A => ("inc a".to_owned(), 1),
        JMP => (format!("jmp {}", format_word(op_lo, op_hi)), 3),
        LD_A => (format!("ld a, {}", format_byte(op_lo)), 2),
        TRAP => (format!("trap {}", format_byte(op_lo)), 2),
        _ => ("???".to_owned(), 1),
    }
}

fn format_byte(maybe_byte: Option<&u8>) -> String {
    match maybe_byte {
        Some(byte) => format!("{byte}"),
        None => "???".to_owned(),
    }
}

fn format_word(maybe_lo: Option<&u8>, maybe_hi: Option<&u8>) -> String {
    match (maybe_lo, maybe_hi) {
        (Some(&lo), Some(&hi)) => format!("{}", u16::from_le_bytes([lo, hi])),
        _ => "???".to_owned(),
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

    #[test]
    fn bra_always_branches() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[BRA, 1, HALT, HALT]).unwrap();
        assert_eq!(cpu.pc, 4, "wrong PC");
    }

    #[test]
    fn beq_takes_branch_if_zero_flag_set() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 0, BEQ, 1, HALT, HALT]).unwrap();
        assert_eq!(cpu.pc, 6, "wrong PC");
    }

    #[test]
    fn beq_does_not_take_branch_if_zero_flag_clear() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 1, BEQ, 1, HALT, HALT]).unwrap();
        assert_eq!(cpu.pc, 5, "wrong PC");
    }

    #[test]
    fn cmp_sets_zero_if_operand_equal() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 1, CMP_A, 1, HALT]).unwrap();
        assert_eq!(cpu.zero, true, "zero not set");
    }

    #[test]
    fn cmp_clears_zero_if_operand_not_equal() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 0, CMP_A, 1, HALT]).unwrap();
        assert_eq!(cpu.zero, false, "zero not cleared");
    }

    #[test]
    fn disassemble_correctly_disassembles_code() {
        struct Case {
            code: &'static [u8],
            source: &'static str,
        }
        let cases: &[Case] = &[
            Case {
                code: &[BEQ, 3],
                source: "beq 3",
            },
            Case {
                code: &[CMP_A, 1],
                source: "cmp a, 1",
            },
            Case {
                code: &[DEC_A],
                source: "dec a",
            },
            Case {
                code: &[HALT],
                source: "halt",
            },
            Case {
                code: &[INC_A],
                source: "inc a",
            },
            Case {
                code: &[JMP, 2, 0],
                source: "jmp 2",
            },
            Case {
                code: &[LD_A, 10],
                source: "ld a, 10",
            },
            Case {
                code: &[TRAP, 32],
                source: "trap 32",
            },
            Case {
                code: &[255],
                source: "???",
            },
        ];
        for case in cases {
            let (code, bytes) = disassemble(case.code);
            assert_eq!(code, case.source, "wrong disassembly: {:?}", case.code);
            assert_eq!(bytes, case.code.len(), "wrong length");
        }
    }
}
