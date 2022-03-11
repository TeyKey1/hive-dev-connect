extern crate anyhow;
extern crate clap;
extern crate clap_verbosity_flag;
extern crate colored;
extern crate ll_api;
extern crate log;
extern crate rppal;
extern crate simple_clap_logger;

use std::convert::TryFrom;
use std::ops::RangeInclusive;
use std::process::exit;
use std::str::FromStr;
use std::sync::Mutex;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use ll_api::{Target, TargetStackShield, TestChannel};
use log::error;
use log::Level;
use pca9535::IoExpander;
use pca9535::Pca9535Immediate;
use rppal::i2c::I2c;
use simple_clap_logger::Logger;

const TSS_BASE_ADDR: u8 = 32;
const TSS_RANGE: RangeInclusive<u8> = RangeInclusive::new(0, 7);
const CHANNEL_RANGE: RangeInclusive<u8> = RangeInclusive::new(0, 3);

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Position of the TSS
    #[clap(default_value_t = 0, validator = validate_tss)]
    tss: u8,
    /// Test channel number
    #[clap(default_value_t = 0, validator = validate_test_channel)]
    test_ch: u8,
    #[clap(default_value_t = 0, validator = validate_target)]
    /// Target number
    target_ch: u8,
    /// Disconnects all connections on TSS
    #[clap(short, long)]
    disconnect: bool,
    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() {
    let args = Args::parse();
    Logger::init_with_level(set_log_level(&args.verbose.log_level()));

    let res = app(args);

    if let Err(err) = res {
        error!("{}", err);
        exit(1);
    }
}

fn app(args: Args) -> Result<()> {
    let i2c = I2c::new()?;

    let expander = Pca9535Immediate::new(i2c, TSS_BASE_ADDR + args.tss);
    let io_expander: IoExpander<Mutex<_>, _> = IoExpander::new(expander);

    let mut shield = TargetStackShield::new(&io_expander);

    shield.init_pins()?;

    shield.disconnect_all()?;

    if args.disconnect {
        println!(
            "{} {}",
            "Sucessfully disconnected all connections on TSS".green(),
            args.tss.to_string().magenta()
        );
        return Ok(());
    }

    if !shield.daughterboard_is_connected()? {
        error!("No daughterboard detected on selected TSS");
        exit(1);
    }

    shield.connect_test_channel_to_target(
        TestChannel::try_from(args.test_ch)?,
        Target::try_from(args.target_ch)?,
    )?;

    println!(
        "{} {} {} {}",
        "Successfully connected test channel".green(),
        args.test_ch.to_string().magenta(),
        "to target".green(),
        args.target_ch.to_string().magenta()
    );
    Ok(())
}

/// set the log level of the cli
fn set_log_level(verbosity: &Option<log::Level>) -> Level {
    match verbosity {
        Some(level) => *level,
        None => Level::Error,
    }
}

fn validate_tss(s: &str) -> Result<(), String> {
    u8::from_str(s)
        .map(|val| TSS_RANGE.contains(&val))
        .map_err(|err| err.to_string())
        .and_then(|res| match res {
            true => Ok(()),
            false => Err(format!(
                "TSS not in valid range {} - {}",
                TSS_RANGE.start(),
                TSS_RANGE.end()
            )),
        })
}

fn validate_target(s: &str) -> Result<(), String> {
    u8::from_str(s)
        .map(|val| CHANNEL_RANGE.contains(&val))
        .map_err(|err| err.to_string())
        .and_then(|res| match res {
            true => Ok(()),
            false => Err(format!(
                "Target not in valid range {} - {}",
                CHANNEL_RANGE.start(),
                CHANNEL_RANGE.end()
            )),
        })
}

fn validate_test_channel(s: &str) -> Result<(), String> {
    u8::from_str(s)
        .map(|val| CHANNEL_RANGE.contains(&val))
        .map_err(|err| err.to_string())
        .and_then(|res| match res {
            true => Ok(()),
            false => Err(format!(
                "Test channel not in valid range {} - {}",
                CHANNEL_RANGE.start(),
                CHANNEL_RANGE.end()
            )),
        })
}
