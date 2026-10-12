use anyhow::Result;

use std::{
    env,
    io::{Write as _, stdin, stdout},
    num::ParseIntError,
};

use r8::Cpu;

fn main() -> Result<()> {
    let program: Result<Vec<u8>, ParseIntError> =
        env::args().skip(1).map(|arg| arg.parse()).collect();
    let mut cpu = Cpu::default();
    cpu.load_program(&program?)?;
    let mut buffer = String::new();
    println!("  PC  A");
    loop {
        print!("{:04} {:02} >", cpu.pc, cpu.a);
        stdout().flush()?;
        stdin().read_line(&mut buffer)?;
        cpu.step();
    }
}
