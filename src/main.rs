extern crate ll_api;
extern crate rppal;
extern crate shared_bus;

use std::sync::Mutex;

use ll_api::TargetStackShield;
use ll_api::{Probe, Target};
use pca9535::IoExpander;
use pca9535::Pca9535Immediate;
use rppal::i2c::I2c;

const ADDR: u8 = 32;

fn main() {
    let i2c = I2c::new().unwrap();

    let i2c_bus: &'static _ = shared_bus::new_std!(I2c = i2c).unwrap();

    let expander = Pca9535Immediate::new(i2c_bus.acquire_i2c(), ADDR);
    let io_expander: IoExpander<Mutex<_>, _> = IoExpander::new(expander);

    let mut shield = TargetStackShield::new(&io_expander);

    shield.init_pins().unwrap();

    if shield.daughterboard_is_connected().unwrap() {
        shield
            .connect_probe_to_target(Probe::Probe0, Target::Target0)
            .unwrap();
    }
}
