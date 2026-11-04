use anyhow::{Context as _, Result};

pub const ADD_A: u8 = 0x50;
pub const BEQ: u8 = 0xF1;
pub const BRA: u8 = 0xF0;
pub const CMP_A: u8 = 0x70;
pub const DEC_A: u8 = 0x40;
pub const HALT: u8 = 0x00;
pub const INC_A: u8 = 0x30;
pub const JMP: u8 = 0xF7;
pub const LD_A: u8 = 0x10;
pub const TRAP: u8 = 0xF9;

pub const TRAP_PRINT: u8 = 0x20;

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
            a: 0x00,
            mem: [0x00; 65536],
            pc: 0x0000,
            zero: false,
        }
    }
}

impl Cpu {
    pub fn fetch(&mut self) -> u8 {
        let address = usize::from(self.pc);
        let value = self.mem.get(address).copied().unwrap_or_default();
        self.pc = self.pc.wrapping_add(0x01);
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
                self.a = self.a.wrapping_sub(0x01);
                self.update_zero(self.a);
            }
            HALT => return false,
            INC_A => {
                self.a = self.a.wrapping_add(0x01);
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
                if operand == TRAP_PRINT {
                    print!("{}", char::from(self.a));
                }
            }
            _ => {}
        }
        true
    }

    pub fn update_zero(&mut self, value: u8) {
        self.zero = value == 0x00;
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
        Some(byte) => format!("{byte:#04X}"),
        None => "???".to_owned(),
    }
}

fn format_word(maybe_lo: Option<&u8>, maybe_hi: Option<&u8>) -> String {
    match (maybe_lo, maybe_hi) {
        (Some(&lo), Some(&hi)) => format!("{:#06X}", u16::from_le_bytes([lo, hi])),
        _ => "???".to_owned(),
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests")]
mod tests {
    use super::*;

    macro_rules! assert_eq_hex {
        ( $got:expr, $want:expr, $msg:expr ) => {
            assert_eq!(
                $got, $want,
                "{}: want {:#04X}, got {:#04X}",
                $msg, $want, $got,
            );
        };
    }

    macro_rules! assert_eq_hex16 {
        ( $got:expr, $want:expr, $msg:expr ) => {
            assert_eq!(
                $got, $want,
                "{}: want {:#06X}, got {:#06X}",
                $msg, $want, $got,
            );
        };
    }

    macro_rules! assert_zero {
        ( $got:expr, $want:expr ) => {
            assert_eq!($got, $want, "want zero {}, got {}", $want, $got,);
        };
    }

    #[test]
    fn default_correctly_initialises_cpu() {
        let cpu = Cpu::default();
        assert_eq_hex!(cpu.a, 0x00, "wrong A");
        assert_eq_hex16!(cpu.pc, 0x0000, "wrong PC");
        assert_zero!(cpu.zero, false);
        assert_eq!(*cpu.mem.first().unwrap(), 0, "wrong memory contents");
    }

    #[test]
    fn step_increments_pc() {
        let mut cpu = Cpu::default();
        cpu.step();
        assert_eq_hex16!(cpu.pc, 0x0001, "wrong PC");
        cpu.step();
        assert_eq_hex16!(cpu.pc, 0x0002, "wrong PC");
    }

    #[test]
    fn inc_increments_a() {
        let mut cpu = Cpu::default();
        cpu.zero = true;
        cpu.run_program(&[INC_A, HALT]).unwrap();
        assert_eq_hex!(cpu.a, 0x01, "wrong A");
        assert_zero!(cpu.zero, false);
    }

    #[test]
    fn a_goes_from_ff_to_00() {
        let mut cpu = Cpu::default();
        cpu.a = 0xFF;
        cpu.run_program(&[INC_A, HALT]).unwrap();
        assert_eq_hex!(cpu.a, 0x00, "wrong A");
        assert_zero!(cpu.zero, true);
    }

    #[test]
    fn a_decrements_from_00_to_ff() {
        let mut cpu = Cpu::default();
        cpu.a = 0x00;
        cpu.zero = true;
        cpu.run_program(&[DEC_A, HALT]).unwrap();
        assert_eq_hex!(cpu.a, 0xFF, "wrong A");
        assert_zero!(cpu.zero, false);
    }

    #[test]
    fn dec_sets_zero_flag() {
        let mut cpu = Cpu::default();
        cpu.a = 0x01;
        cpu.run_program(&[DEC_A, HALT]).unwrap();
        assert_zero!(cpu.zero, true);
    }

    #[test]
    fn pc_goes_from_ffff_to_0000() {
        let mut cpu = Cpu::default();
        cpu.pc = 0xFFFF;
        cpu.step();
        assert_eq_hex16!(cpu.pc, 0x0000, "wrong PC");
    }

    #[test]
    fn memory_is_bytes() {
        let mut cpu = Cpu::default();
        cpu.mem[0x0000] = 0x00;
        assert_eq!(cpu.mem[0x0000], 0x00, "wrong memory contents");
    }

    #[test]
    fn run_runs_until_halted() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[INC_A, HALT]).unwrap();
        assert_eq_hex!(cpu.a, 0x01, "wrong A");
        assert_eq_hex16!(cpu.pc, 0x0002, "wrong PC");
    }

    #[test]
    fn ld_loads_accumulator() {
        let mut cpu = Cpu::default();
        cpu.a = 0x0A;
        cpu.zero = true;
        cpu.run_program(&[LD_A, 0x05, HALT]).unwrap();
        assert_eq_hex!(cpu.a, 0x05, "wrong A");
        assert_eq_hex16!(cpu.pc, 0x0003, "wrong PC");
        assert_zero!(cpu.zero, false);
    }

    #[test]
    fn ld_sets_zero_flag() {
        let mut cpu = Cpu::default();
        cpu.a = 0x0A;
        cpu.run_program(&[LD_A, 0x00, HALT]).unwrap();
        assert_zero!(cpu.zero, true);
    }

    #[test]
    fn add_adds_to_accumulator() {
        let mut cpu = Cpu::default();
        cpu.a = 0x01;
        cpu.zero = true;
        cpu.run_program(&[ADD_A, 0x02, HALT]).unwrap();
        assert_eq_hex!(cpu.a, 0x03, "wrong A");
        assert_eq_hex16!(cpu.pc, 0x0003, "wrong PC");
        assert_zero!(cpu.zero, false);
    }

    #[test]
    fn add_sets_zero_flag() {
        let mut cpu = Cpu::default();
        cpu.a = 0x01;
        cpu.run_program(&[ADD_A, 0xFF, HALT]).unwrap();
        assert_eq!(cpu.zero, true, "zero not set");
    }

    #[test]
    fn jmp_jumps_to_specified_address() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[JMP, 0x05, 0x00, INC_A, HALT, HALT])
            .unwrap();
        assert_eq_hex!(cpu.a, 0x00, "wrong A");
        assert_eq_hex16!(cpu.pc, 0x0006, "wrong PC");
    }

    #[test]
    fn bra_always_branches() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[BRA, 0x01, HALT, HALT]).unwrap();
        assert_eq_hex16!(cpu.pc, 0x0004, "wrong PC");
    }

    #[test]
    fn beq_takes_branch_if_zero_flag_set() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 0x00, BEQ, 0x01, HALT, HALT])
            .unwrap();
        assert_eq_hex16!(cpu.pc, 0x0006, "wrong PC");
    }

    #[test]
    fn beq_does_not_take_branch_if_zero_flag_clear() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 0x01, BEQ, 0x01, HALT, HALT])
            .unwrap();
        assert_eq_hex16!(cpu.pc, 0x0005, "wrong PC");
    }

    #[test]
    fn cmp_sets_zero_if_operand_equal() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 0x01, CMP_A, 0x01, HALT]).unwrap();
        assert_zero!(cpu.zero, true);
    }

    #[test]
    fn cmp_clears_zero_if_operand_not_equal() {
        let mut cpu = Cpu::default();
        cpu.run_program(&[LD_A, 0x00, CMP_A, 0x01, HALT]).unwrap();
        assert_zero!(cpu.zero, false);
    }

    #[test]
    fn disassemble_correctly_disassembles_code() {
        struct Case {
            code: &'static [u8],
            source: &'static str,
        }
        let cases: &[Case] = &[
            Case {
                code: &[BEQ, 0x03],
                source: "beq 0x03",
            },
            Case {
                code: &[CMP_A, 1],
                source: "cmp a, 0x01",
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
                code: &[JMP, 0x02, 0x00],
                source: "jmp 0x0002",
            },
            Case {
                code: &[LD_A, 0x0A],
                source: "ld a, 0x0A",
            },
            Case {
                code: &[TRAP, 0x20],
                source: "trap 0x20",
            },
            Case {
                code: &[0xFF],
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
