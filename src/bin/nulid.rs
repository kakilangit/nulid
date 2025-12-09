//! NULID CLI - Command-line interface for NULID generation and manipulation

use std::fmt::Write;
use std::io::{self, BufRead};
use std::process;

use nulid::Nulid;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_help();
        process::exit(0);
    }

    match args[1].as_str() {
        "generate" | "gen" | "g" => {
            let count = if args.len() > 2 {
                args[2].parse::<usize>().unwrap_or_else(|_| {
                    eprintln!("Error: Invalid count '{}'", args[2]);
                    process::exit(1);
                })
            } else {
                1
            };
            generate(count);
        }
        "parse" | "p" => {
            if args.len() < 3 {
                eprintln!("Error: NULID string required for parse command");
                eprintln!("Usage: nulid parse <nulid-string>");
                process::exit(1);
            }
            parse(&args[2]);
        }
        "inspect" | "i" => {
            if args.len() < 3 {
                eprintln!("Error: NULID string required for inspect command");
                eprintln!("Usage: nulid inspect <nulid-string>");
                process::exit(1);
            }
            inspect(&args[2]);
        }
        "decode" | "d" => {
            if args.len() < 3 {
                eprintln!("Error: NULID string required for decode command");
                eprintln!("Usage: nulid decode <nulid-string>");
                process::exit(1);
            }
            decode(&args[2]);
        }
        "validate" | "v" => {
            if args.len() > 2 {
                validate_args(&args[2..]);
            } else {
                validate_stdin();
            }
        }
        "help" | "-h" | "--help" => {
            print_help();
        }
        "version" | "-v" | "--version" => {
            println!("nulid {}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", args[1]);
            eprintln!();
            print_help();
            process::exit(1);
        }
    }
}

fn generate(count: usize) {
    for _ in 0..count {
        match Nulid::new() {
            Ok(nulid) => println!("{nulid}"),
            Err(e) => {
                eprintln!("Error generating NULID: {e}");
                process::exit(1);
            }
        }
    }
}

fn parse(nulid_str: &str) {
    match nulid_str.parse::<Nulid>() {
        Ok(nulid) => {
            println!("{nulid}");
        }
        Err(e) => {
            eprintln!("Error parsing NULID: {e}");
            process::exit(1);
        }
    }
}

fn inspect(nulid_str: &str) {
    match nulid_str.parse::<Nulid>() {
        Ok(nulid) => {
            let nanos = nulid.timestamp_nanos();
            let random = nulid.random();
            let bytes = nulid.to_bytes();
            let datetime = nulid.datetime();

            println!("NULID:       {nulid}");
            println!("Timestamp:   {nanos} ns since epoch");
            println!("Seconds:     {} s", nulid.seconds());
            println!("Subsec:      {} ns", nulid.subsec_nanos());
            println!("Randomness:  {random} (60-bit)");
            println!("Bytes:       {}", hex_encode(&bytes));
            println!("DateTime:    {datetime:?}");
            println!("u128 value:  0x{:032X}", nulid.as_u128());
        }
        Err(e) => {
            eprintln!("Error inspecting NULID: {e}");
            process::exit(1);
        }
    }
}

fn decode(nulid_str: &str) {
    match nulid_str.parse::<Nulid>() {
        Ok(nulid) => {
            let bytes = nulid.to_bytes();
            println!("{}", hex_encode(&bytes));
        }
        Err(e) => {
            eprintln!("Error decoding NULID: {e}");
            process::exit(1);
        }
    }
}

fn validate_args(nulid_strs: &[String]) {
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for nulid_str in nulid_strs {
        match nulid_str.parse::<Nulid>() {
            Ok(_) => {
                println!("{nulid_str}: valid");
                valid_count += 1;
            }
            Err(e) => {
                println!("{nulid_str}: invalid ({e})");
                invalid_count += 1;
            }
        }
    }

    println!();
    println!("Valid:   {valid_count}");
    println!("Invalid: {invalid_count}");

    if invalid_count > 0 {
        process::exit(1);
    }
}

fn validate_stdin() {
    let stdin = io::stdin();
    let mut valid_count = 0;
    let mut invalid_count = 0;

    for line in stdin.lock().lines() {
        match line {
            Ok(nulid_str) => {
                let trimmed = nulid_str.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match trimmed.parse::<Nulid>() {
                    Ok(_) => {
                        println!("{trimmed}: valid");
                        valid_count += 1;
                    }
                    Err(e) => {
                        println!("{trimmed}: invalid ({e})");
                        invalid_count += 1;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading stdin: {e}");
                process::exit(1);
            }
        }
    }

    println!();
    println!("Valid:   {valid_count}");
    println!("Invalid: {invalid_count}");

    if invalid_count > 0 {
        process::exit(1);
    }
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut output, b| {
        let _ = write!(output, "{b:02x}");
        output
    })
}

fn print_help() {
    println!("NULID CLI - Nanosecond-Precision Universally Lexicographically Sortable Identifier");
    println!();
    println!("USAGE:");
    println!("    nulid <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    generate, gen, g [COUNT]    Generate NULID(s) (default: 1)");
    println!("    parse, p <NULID>            Parse and validate a NULID string");
    println!("    inspect, i <NULID>          Inspect NULID components in detail");
    println!("    decode, d <NULID>           Decode NULID to hex bytes");
    println!("    validate, v [NULID...]      Validate NULID(s) from args or stdin");
    println!("    help, -h, --help            Print this help message");
    println!("    version, -v, --version      Print version information");
    println!();
    println!("EXAMPLES:");
    println!("    # Generate a single NULID");
    println!("    nulid generate");
    println!();
    println!("    # Generate 10 NULIDs");
    println!("    nulid gen 10");
    println!();
    println!("    # Parse a NULID string");
    println!("    nulid parse 01GZWQ22K2MNDR0GAQTE834QRV");
    println!();
    println!("    # Inspect NULID details");
    println!("    nulid inspect 01GZWQ22K2MNDR0GAQTE834QRV");
    println!();
    println!("    # Decode to hex");
    println!("    nulid decode 01GZWQ22K2MNDR0GAQTE834QRV");
    println!();
    println!("    # Validate multiple NULIDs");
    println!("    nulid validate 01GZWQ22K2MNDR0GAQTE834QRV 01GZWQ22K2TKVGHH1Z1G0AK1EK");
    println!();
    println!("    # Validate from stdin");
    println!("    cat nulids.txt | nulid validate");
    println!();
    println!("For more information, visit: https://github.com/kakilangit/nulid");
}
