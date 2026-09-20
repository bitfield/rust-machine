use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use r8::Cpu;

fn main() -> Result<()> {
    let mut cpu = Cpu::default();
    let mut buffer = String::new();
    println!("  PC  A");
    loop {
        print!("{:04} {:02} >", cpu.pc, cpu.a);
        stdout().flush()?;
        stdin().read_line(&mut buffer)?;
        cpu.step();
    }
}
