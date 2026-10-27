use anyhow::Result;
use r8::disassemble;

use std::{env, fs};

const USAGE: &str = "Usage: disasm FILE";

#[expect(clippy::unwrap_used, reason = "arg count checked")]
fn main() -> Result<()> {
    if env::args().count() < 2 {
        println!("{USAGE}");
        return Ok(());
    }
    let path = env::args().nth(1).unwrap();
    let program = fs::read(path)?;
    let mut i: usize = 0;
    loop {
        let code = program.get(i..).unwrap_or_default();
        if code.is_empty() {
            break;
        }
        let (source, bytes) = disassemble(code);
        println!("{source}");
        i = i.strict_add(bytes);
    }
    Ok(())
}
