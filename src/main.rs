extern crate clap;
extern crate ll_api;
extern crate rppal;
extern crate anyhow;

use std::convert::TryFrom;
use std::sync::Mutex;

use clap::Parser;
use anyhow::{Result, anyhow};
use ll_api::{TargetStackShield, TestChannel, Target};
use pca9535::IoExpander;
use pca9535::Pca9535Immediate;
use rppal::i2c::I2c;

const TSS_BASE_ADDR: u8= 32;

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
}

fn main() -> Result<()> {
    let args = Args::parse();

    let i2c = I2c::new()?;

    let expander = Pca9535Immediate::new(i2c, TSS_BASE_ADDR + args.tss);
    let io_expander: IoExpander<Mutex<_>, _> = IoExpander::new(expander);

    let mut shield = TargetStackShield::new(&io_expander);

    shield.init_pins()?;

    shield.disconnect_all()?;

    if args.disconnect {
        return Ok(());
    }

    if !shield.daughterboard_is_connected()? {
        return Err(anyhow!("No daughterbuard detected on selected TSS"));
    }

    shield.connect_test_channel_to_target(TestChannel::try_from(args.test_ch)?, Target::try_from(args.target_ch)?)?;
    
    Ok(())
}
