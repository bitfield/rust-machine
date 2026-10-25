use anyhow::Result;

use std::{
    env,
    io::{Write as _, stdin, stdout},
    num::ParseIntError,
};

use r8::{Cpu, disassemble};

fn main() -> Result<()> {
    let program: Result<Vec<u8>, ParseIntError> =
        env::args().skip(1).map(|arg| arg.parse()).collect();
    let mut cpu = Cpu::default();
    cpu.load_program(&program?)?;
    let mut buffer = String::new();
    loop {
        let addr = usize::from(cpu.pc);
        let code = cpu.mem.get(addr..addr + 3).unwrap_or_default();
        let asm = disassemble(code);
        println!("  PC  A | NEXT");
        print!("{:04} {:02} | {asm}\n> ", cpu.pc, cpu.a);
        stdout().flush()?;
        stdin().read_line(&mut buffer)?;
        cpu.step();
    }
}
