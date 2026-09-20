#[expect(clippy::min_ident_chars, reason = "some regs have single-letter names")]
#[derive(Debug)]
pub struct Cpu {
    pub a: usize,
    pub mem: [usize; 65536],
    pub pc: usize,
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
    pub fn step(&mut self) {
        let opcode = self.mem.get(self.pc).copied().unwrap_or_default();
        self.pc = self.pc.wrapping_add(1);
        if opcode == 48 {
            // `inc`
            self.a = self.a.wrapping_add(1);
        }
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
        cpu.mem[0] = 48; // `inc`
        cpu.step();
        assert_eq!(cpu.a, 1, "wrong a after `inc`");
    }
}
