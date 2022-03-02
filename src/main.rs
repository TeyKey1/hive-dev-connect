extern crate clap;
extern crate ll_api;
extern crate rppal;

use core::panic;
use std::convert::TryFrom;
use std::sync::Mutex;

use clap::Parser;
use ll_api::{TargetStackShield, TestChannel, Target};
use pca9535::IoExpander;
use pca9535::Pca9535Immediate;
use rppal::i2c::I2c;

const TSS_BASE_ADDR: u8= 32;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(default_value_t = 0)]
    tss: u8,
    #[clap(default_value_t = 0)]
    test_ch: u8,
    #[clap(default_value_t = 0)]
    target_ch: u8,
    #[clap(short, long)]
    disconnect: bool,
}

fn main() {
    let args = Args::parse();

    let i2c = I2c::new().unwrap();

    let expander = Pca9535Immediate::new(i2c, TSS_BASE_ADDR + args.tss);
    let io_expander: IoExpander<Mutex<_>, _> = IoExpander::new(expander);

    let mut shield = TargetStackShield::new(&io_expander);

    shield.init_pins().unwrap();

    shield.disconnect_all().unwrap();

    if args.disconnect {
        return ()
    }

    if !shield.daughterboard_is_connected().unwrap() {
        panic!("No daughterboard connected to specified TSS");
    }

    shield.connect_test_channel_to_target(TestChannel::try_from(args.test_ch).unwrap(), Target::try_from(args.target_ch).unwrap()).unwrap();
}
