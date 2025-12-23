//! NULID CLI - Command-line interface for NULID generation and manipulation

use std::fmt::Write;
use std::io::{self, BufRead};
use std::process;

use nulid::Nulid;

#[cfg(feature = "uuid")]
use uuid::Uuid;

#[cfg(feature = "chrono")]
use chrono::{DateTime, Utc};

#[allow(clippy::too_many_lines)]
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
        "uuid" | "u" => {
            #[cfg(feature = "uuid")]
            {
                if args.len() < 3 {
                    eprintln!("Error: NULID string required for uuid command");
                    eprintln!("Usage: nulid uuid <nulid-string>");
                    process::exit(1);
                }
                to_uuid(&args[2]);
            }
            #[cfg(not(feature = "uuid"))]
            {
                eprintln!("Error: uuid feature not enabled");
                eprintln!("Rebuild with: cargo build --features uuid");
                process::exit(1);
            }
        }
        "from-uuid" | "fu" => {
            #[cfg(feature = "uuid")]
            {
                if args.len() < 3 {
                    eprintln!("Error: UUID string required for from-uuid command");
                    eprintln!("Usage: nulid from-uuid <uuid-string>");
                    process::exit(1);
                }
                from_uuid(&args[2]);
            }
            #[cfg(not(feature = "uuid"))]
            {
                eprintln!("Error: uuid feature not enabled");
                eprintln!("Rebuild with: cargo build --features uuid");
                process::exit(1);
            }
        }
        "datetime" | "dt" => {
            #[cfg(feature = "chrono")]
            {
                if args.len() < 3 {
                    eprintln!("Error: NULID string required for datetime command");
                    eprintln!("Usage: nulid datetime <nulid-string>");
                    process::exit(1);
                }
                to_datetime(&args[2]);
            }
            #[cfg(not(feature = "chrono"))]
            {
                eprintln!("Error: chrono feature not enabled");
                eprintln!("Rebuild with: cargo build --features chrono");
                process::exit(1);
            }
        }
        "from-datetime" | "fdt" => {
            #[cfg(feature = "chrono")]
            {
                if args.len() < 3 {
                    eprintln!("Error: ISO 8601 datetime string required for from-datetime command");
                    eprintln!("Usage: nulid from-datetime <iso8601-datetime>");
                    process::exit(1);
                }
                from_datetime(&args[2]);
            }
            #[cfg(not(feature = "chrono"))]
            {
                eprintln!("Error: chrono feature not enabled");
                eprintln!("Rebuild with: cargo build --features chrono");
                process::exit(1);
            }
        }
        "compare" | "cmp" | "c" => {
            if args.len() < 4 {
                eprintln!("Error: Two NULID strings required for compare command");
                eprintln!("Usage: nulid compare <nulid1> <nulid2>");
                process::exit(1);
            }
            compare(&args[2], &args[3]);
        }
        "sort" | "s" => {
            if args.len() > 2 {
                sort_args(&args[2..]);
            } else {
                sort_stdin();
            }
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
            let nanos = nulid.nanos();
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

            #[cfg(feature = "uuid")]
            {
                let uuid = nulid.to_uuid();
                println!("UUID:        {uuid}");
            }

            #[cfg(feature = "chrono")]
            {
                let chrono_dt = nulid.chrono_datetime();
                println!("Chrono DT:   {chrono_dt}");
            }
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

#[cfg(feature = "uuid")]
fn to_uuid(nulid_str: &str) {
    match nulid_str.parse::<Nulid>() {
        Ok(nulid) => {
            let uuid = nulid.to_uuid();
            println!("{uuid}");
        }
        Err(e) => {
            eprintln!("Error parsing NULID: {e}");
            process::exit(1);
        }
    }
}

#[cfg(feature = "uuid")]
fn from_uuid(uuid_str: &str) {
    match uuid_str.parse::<Uuid>() {
        Ok(uuid) => {
            let nulid = Nulid::from_uuid(uuid);
            println!("{nulid}");
        }
        Err(e) => {
            eprintln!("Error parsing UUID: {e}");
            process::exit(1);
        }
    }
}

#[cfg(feature = "chrono")]
fn to_datetime(nulid_str: &str) {
    match nulid_str.parse::<Nulid>() {
        Ok(nulid) => {
            let dt = nulid.chrono_datetime();
            println!("{}", dt.to_rfc3339());
        }
        Err(e) => {
            eprintln!("Error parsing NULID: {e}");
            process::exit(1);
        }
    }
}

#[cfg(feature = "chrono")]
fn from_datetime(datetime_str: &str) {
    match datetime_str.parse::<DateTime<Utc>>() {
        Ok(dt) => match Nulid::from_chrono_datetime(dt) {
            Ok(nulid) => println!("{nulid}"),
            Err(e) => {
                eprintln!("Error creating NULID: {e}");
                process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error parsing datetime: {e}");
            eprintln!("Expected ISO 8601 format, e.g., 2024-01-01T00:00:00Z");
            process::exit(1);
        }
    }
}

fn compare(nulid_str1: &str, nulid_str2: &str) {
    let nulid1 = match nulid_str1.parse::<Nulid>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error parsing first NULID: {e}");
            process::exit(1);
        }
    };

    let nulid2 = match nulid_str2.parse::<Nulid>() {
        Ok(n) => n,
        Err(e) => {
            eprintln!("Error parsing second NULID: {e}");
            process::exit(1);
        }
    };

    println!("NULID 1:     {nulid1}");
    println!("  Timestamp: {} ns", nulid1.nanos());
    println!("  Random:    {}", nulid1.random());
    println!();
    println!("NULID 2:     {nulid2}");
    println!("  Timestamp: {} ns", nulid2.nanos());
    println!("  Random:    {}", nulid2.random());
    println!();

    match nulid1.cmp(&nulid2) {
        std::cmp::Ordering::Less => {
            println!("Result:      NULID 1 < NULID 2 (earlier)");
            let diff = nulid2.nanos().saturating_sub(nulid1.nanos());
            println!("Time diff:   {diff} ns");
        }
        std::cmp::Ordering::Equal => {
            println!("Result:      NULID 1 == NULID 2 (equal)");
        }
        std::cmp::Ordering::Greater => {
            println!("Result:      NULID 1 > NULID 2 (later)");
            let diff = nulid1.nanos().saturating_sub(nulid2.nanos());
            println!("Time diff:   {diff} ns");
        }
    }
}

fn sort_args(nulid_strs: &[String]) {
    let mut nulids: Vec<(String, Nulid)> = Vec::new();

    for nulid_str in nulid_strs {
        match nulid_str.parse::<Nulid>() {
            Ok(nulid) => {
                nulids.push((nulid_str.clone(), nulid));
            }
            Err(e) => {
                eprintln!("Error parsing NULID '{nulid_str}': {e}");
                process::exit(1);
            }
        }
    }

    nulids.sort_by_key(|(_, nulid)| *nulid);

    for (original, _) in nulids {
        println!("{original}");
    }
}

fn sort_stdin() {
    let stdin = io::stdin();
    let mut nulids: Vec<(String, Nulid)> = Vec::new();

    for line in stdin.lock().lines() {
        match line {
            Ok(nulid_str) => {
                let trimmed = nulid_str.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match trimmed.parse::<Nulid>() {
                    Ok(nulid) => {
                        nulids.push((trimmed.to_string(), nulid));
                    }
                    Err(e) => {
                        eprintln!("Error parsing NULID '{trimmed}': {e}");
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error reading stdin: {e}");
                process::exit(1);
            }
        }
    }

    nulids.sort_by_key(|(_, nulid)| *nulid);

    for (original, _) in nulids {
        println!("{original}");
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
    println!("    generate, gen, g [COUNT]       Generate NULID(s) (default: 1)");
    println!("    parse, p <NULID>               Parse and validate a NULID string");
    println!("    inspect, i <NULID>             Inspect NULID components in detail");
    println!("    decode, d <NULID>              Decode NULID to hex bytes");
    println!("    validate, v [NULID...]         Validate NULID(s) from args or stdin");
    println!("    compare, cmp, c <N1> <N2>      Compare two NULIDs");
    println!("    sort, s [NULID...]             Sort NULIDs from args or stdin");
    println!();
    #[cfg(feature = "uuid")]
    println!("UUID COMMANDS (requires --features uuid):");
    #[cfg(not(feature = "uuid"))]
    println!("UUID COMMANDS (disabled - rebuild with --features uuid):");
    println!("    uuid, u <NULID>                Convert NULID to UUID");
    println!("    from-uuid, fu <UUID>           Convert UUID to NULID");
    println!();
    #[cfg(feature = "chrono")]
    println!("DATETIME COMMANDS (requires --features chrono):");
    #[cfg(not(feature = "chrono"))]
    println!("DATETIME COMMANDS (disabled - rebuild with --features chrono):");
    println!("    datetime, dt <NULID>           Convert NULID to ISO 8601 datetime");
    println!("    from-datetime, fdt <DATETIME>  Create NULID from ISO 8601 datetime");
    println!();
    println!("OTHER COMMANDS:");
    println!("    help, -h, --help               Print this help message");
    println!("    version, -v, --version         Print version information");
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
    println!("    # Compare two NULIDs");
    println!("    nulid compare 01GZWQ22K2MNDR0GAQTE834QRV 01GZWQ22K2TKVGHH1Z1G0AK1EK");
    println!();
    println!("    # Sort NULIDs");
    println!("    nulid sort 01GZWQ22K2TKVGHH1Z1G0AK1EK 01GZWQ22K2MNDR0GAQTE834QRV");
    println!();
    println!("    # Sort from stdin");
    println!("    cat nulids.txt | nulid sort");
    println!();
    #[cfg(feature = "uuid")]
    {
        println!("    # Convert NULID to UUID");
        println!("    nulid uuid 01GZWQ22K2MNDR0GAQTE834QRV");
        println!();
        println!("    # Convert UUID to NULID");
        println!("    nulid from-uuid 018d3f9c-5a2e-7b4d-8f1c-3e6a9d2c5b7e");
        println!();
    }
    #[cfg(feature = "chrono")]
    {
        println!("    # Convert NULID to datetime");
        println!("    nulid datetime 01GZWQ22K2MNDR0GAQTE834QRV");
        println!();
        println!("    # Create NULID from datetime");
        println!("    nulid from-datetime 2024-01-01T00:00:00Z");
        println!();
    }
    println!("For more information, visit: https://github.com/kakilangit/nulid");
}
