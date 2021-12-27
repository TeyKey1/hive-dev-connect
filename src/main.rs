extern crate ll_api;
extern crate rppal;
extern crate shared_bus;

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use ll_api::TargetStackShield;
use ll_api::{Probe, Target};
use pca9535::IoExpander;
use pca9535::Pca9535Immediate;
use rppal::i2c::I2c;
use rppal::uart::{Parity, Uart};

const ADDR: u8 = 32;

fn main() {
    let i2c = I2c::new().unwrap();

    let i2c_bus: &'static _ = shared_bus::new_std!(I2c = i2c).unwrap();

    let mut uart = Uart::with_path(Path::new("/dev/ttyAMA3"), 115200, Parity::None, 8, 1).unwrap();
    uart.set_read_mode(1, Duration::from_millis(500)).unwrap();
    uart.set_write_mode(true).unwrap();

    let expander = Pca9535Immediate::new(i2c_bus.acquire_i2c(), ADDR);
    let io_expander: IoExpander<Mutex<_>, _> = IoExpander::new(expander);

    let mut shield = TargetStackShield::new(&io_expander);

    shield.init_pins().unwrap();

    for i in 1..8 {
        let expanderx = Pca9535Immediate::new(i2c_bus.acquire_i2c(), ADDR + i);
        let io_expanderx: IoExpander<Mutex<_>, _> = IoExpander::new(expanderx);

        let mut shieldx = TargetStackShield::new(&io_expanderx);
        shieldx.init_pins().unwrap();
    }

    if shield.daughterboard_is_connected().unwrap() {
        shield
            .connect_probe_to_target(Probe::Probe3, Target::Target0)
            .unwrap();

        let mut buf: [u8; 1] = [0; 1];
        uart.write(&[5]).unwrap();
        uart.read(&mut buf).unwrap();

        println!("received from uart: {}", buf[0]);
    }
}
