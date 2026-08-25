pub const HALT: u8 = 0;
pub const NOP: u8 = 1;

#[derive(Debug)]
pub struct Cpu {
    pub pc: u16,
    pub mem: [u8; 65536],
}

impl Default for Cpu {
    fn default() -> Self {
        Self {
            pc: 0,
            mem: [0; 65536],
        }
    }
}

impl Cpu {
    pub fn step(&mut self) -> bool {
        let opcode = self.mem.get(usize::from(self.pc)).unwrap_or(&0);
        self.pc = self.pc.wrapping_add(1);
        match *opcode {
            HALT => false,
            NOP => true,
            other => unimplemented!("opcode {other}"),
        }
    }

    pub fn run(&mut self) {
        while self.step() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_correctly_initialises_cpu() {
        let cpu = Cpu::default();
        assert_eq!(cpu.pc, 0, "wrong initial PC");
        assert_eq!(*cpu.mem.first().unwrap(), 0, "wrong memory contents");
    }

    #[test]
    fn step_increments_pc() {
        let mut cpu = Cpu::default();
        cpu.mem[256] = NOP;
        cpu.mem[257] = NOP;
        cpu.pc = 256;
        cpu.step();
        assert_eq!(cpu.pc, 257, "wrong PC after first step()");
        cpu.step();
        assert_eq!(cpu.pc, 258, "wrong PC after second step()");
    }

    #[test]
    fn run_runs_until_halted() {
        let mut cpu = Cpu::default();
        cpu.mem[256] = NOP;
        cpu.mem[257] = HALT;
        cpu.pc = 256;
        cpu.run();
        assert_eq!(cpu.pc, 258, "wrong PC after run()");
    }
}
