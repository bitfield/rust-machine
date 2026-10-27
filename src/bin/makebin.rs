use anyhow::Result;

use std::str::FromStr as _;
use std::{env, fs, num::ParseIntError};

const USAGE: &str = "Usage: makebin FILE CODE";

#[expect(clippy::unwrap_used, reason = "arg count checked")]
fn main() -> Result<()> {
    if env::args().count() < 3 {
        println!("{USAGE}");
    }
    let path = env::args().nth(1).unwrap();
    let program: Result<Vec<u8>, ParseIntError> = env::args().skip(2).map(parse_arg).collect();
    fs::write(path, program?)?;
    Ok(())
}

#[expect(clippy::needless_pass_by_value, reason = "convenient for map")]
fn parse_arg(arg: String) -> Result<u8, ParseIntError> {
    if let Some(hex_literal) = arg.strip_prefix("0x") {
        u8::from_str_radix(hex_literal, 16)
    } else {
        u8::from_str(&arg)
    }
}
