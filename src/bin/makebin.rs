use anyhow::Result;

use std::{env, fs, num::ParseIntError};

const USAGE: &str = "Usage: makebin FILE CODE";

#[expect(clippy::unwrap_used, reason = "arg count checked")]
fn main() -> Result<()> {
    if env::args().count() < 3 {
        println!("{USAGE}");
    }
    let path = env::args().nth(1).unwrap();
    let program: Result<Vec<u8>, ParseIntError> =
        env::args().skip(2).map(|arg| arg.parse()).collect();
    fs::write(path, program?)?;
    Ok(())
}
