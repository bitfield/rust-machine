use anyhow::Result;

use std::{
    env, fs,
    io::{Write as _, stdin, stdout},
};

use r8::{Cpu, disassemble};

const USAGE: &str = "Usage: r8 FILE";

#[expect(clippy::unwrap_used, reason = "arg count checked")]
fn main() -> Result<()> {
    if env::args().count() < 2 {
        println!("{USAGE}");
        return Ok(());
    }
    let path = env::args().nth(1).unwrap();
    let program = fs::read(path)?;
    let mut cpu = Cpu::default();
    cpu.load_program(&program)?;
    let mut buffer = String::new();
    loop {
        let addr = usize::from(cpu.pc);
        let code = cpu.mem.get(addr..addr + 3).unwrap_or_default();
        let (asm, _) = disassemble(code);
        println!("  PC  A | NEXT");
        print!("{:04X} {:02X} | {asm}\n> ", cpu.pc, cpu.a);
        stdout().flush()?;
        stdin().read_line(&mut buffer)?;
        if buffer.trim() == "g" {
            cpu.run();
        } else {
            cpu.step();
        }
    }
}
