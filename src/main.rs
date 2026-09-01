use anyhow::Result;

use std::io::{Write as _, stdin, stdout};

use r8::Cpu;

fn main() -> Result<()> {
    let mut cpu = Cpu::default();
    let mut buffer = String::new();
    println!("  PC");
    loop {
        print!("{:04} >", cpu.pc);
        stdout().flush()?;
        stdin().read_line(&mut buffer)?;
        cpu.step();
    }
}
