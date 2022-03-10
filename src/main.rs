extern crate anyhow;
extern crate clap;
extern crate clap_verbosity_flag;
extern crate colored;
extern crate ll_api;
extern crate log;
extern crate simple_clap_logger;
extern crate rppal;

use std::convert::TryFrom;
use std::process::exit;
use std::sync::Mutex;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use ll_api::{Target, TargetStackShield, TestChannel};
use log::Level;
use log::{debug, error, info, trace, warn};
use pca9535::IoExpander;
use pca9535::Pca9535Immediate;
use rppal::i2c::I2c;
use simple_clap_logger::Logger;

const TSS_BASE_ADDR: u8 = 32;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Position of the TSS
    #[clap(default_value_t = 0)]
    tss: u8,
    /// Test channel number
    #[clap(default_value_t = 0)]
    test_ch: u8,
    #[clap(default_value_t = 0)]
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

    info!("starting to process your command :)");
    trace!("tracing..");
    warn!("warning");
    debug!("debugging...");
    error!("error");

    let res = app(args);

    if let Err(err) = res {
        println!("{} {}", "error:".red().bold(), err);
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
