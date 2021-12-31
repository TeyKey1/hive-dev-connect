extern crate ll_api;
extern crate rppal;
extern crate shared_bus;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use ll_api::{RpiTestChannel, TargetStackShield, TestChannel};
use ll_api::{Target, TestInputPin};
use pca9535::IoExpander;
use pca9535::Pca9535Immediate;
use rppal::gpio::Gpio;
use rppal::i2c::I2c;

const ADDR: u8 = 32;

fn main() {
    let i2c = I2c::new().unwrap();
    let mut gpio = Gpio::new().unwrap();

    let i2c_bus: &'static _ = shared_bus::new_std!(I2c = i2c).unwrap();

    let expander = Pca9535Immediate::new(i2c_bus.acquire_i2c(), ADDR);
    let io_expander: IoExpander<Mutex<_>, _> = IoExpander::new(expander);

    let mut shield = TargetStackShield::new(&io_expander);

    shield.init_pins().unwrap();

    let mut test_channels: [Option<RpiTestChannel>; 4] = [None, None, None, None];
    let test_channel_enum = [
        TestChannel::Channel0,
        TestChannel::Channel1,
        TestChannel::Channel2,
        TestChannel::Channel3,
    ];

    let target_enum = [
        Target::Target0,
        Target::Target1,
        Target::Target2,
        Target::Target3,
    ];

    for i in 0..4 {
        test_channels[i] = Some(RpiTestChannel::new(test_channel_enum[i]));

        match test_channels[i] {
            Some(ref mut channel) => channel.init_pins(&mut gpio).unwrap(),
            None => panic!("Terstchannel {} not initialized", i),
        }
    }

    if shield.daughterboard_is_connected().unwrap() {
        for target_n in 0..4 {
            for channel_n in 0..4 {
                shield
                    .connect_test_channel_to_target(
                        test_channel_enum[channel_n],
                        target_enum[target_n],
                    )
                    .unwrap();

                thread::sleep(Duration::from_millis(50));
                let channel = test_channels[channel_n].as_mut().unwrap();

                println!("Target no: {}, channel no: {}", target_n, channel_n);
                println!("Checking gpio initialization");

                assert!(!channel.test_input_is_high(TestInputPin::Pin0).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin1).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin2).unwrap());

                println!("Checking UART connection");

                channel.test_bus_write(&[5]).unwrap();

                assert_eq!(channel.test_bus_read().unwrap()[0], 5);

                println!("Checking Test GPIO");

                channel.test_bus_write(&[0]).unwrap();
                thread::sleep(Duration::from_millis(50));
                assert!(channel.test_input_is_high(TestInputPin::Pin0).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin1).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin2).unwrap());

                channel.test_bus_write(&[10]).unwrap();
                thread::sleep(Duration::from_millis(50));
                assert!(!channel.test_input_is_high(TestInputPin::Pin0).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin1).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin2).unwrap());

                channel.test_bus_write(&[1]).unwrap();
                thread::sleep(Duration::from_millis(50));
                assert!(!channel.test_input_is_high(TestInputPin::Pin0).unwrap());
                assert!(channel.test_input_is_high(TestInputPin::Pin1).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin2).unwrap());

                channel.test_bus_write(&[10]).unwrap();
                thread::sleep(Duration::from_millis(50));
                assert!(!channel.test_input_is_high(TestInputPin::Pin0).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin1).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin2).unwrap());

                channel.test_bus_write(&[2]).unwrap();
                thread::sleep(Duration::from_millis(50));
                assert!(!channel.test_input_is_high(TestInputPin::Pin0).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin1).unwrap());
                assert!(channel.test_input_is_high(TestInputPin::Pin2).unwrap());

                channel.test_bus_write(&[10]).unwrap();
                thread::sleep(Duration::from_millis(50));
                assert!(!channel.test_input_is_high(TestInputPin::Pin0).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin1).unwrap());
                assert!(!channel.test_input_is_high(TestInputPin::Pin2).unwrap());

                channel.test_output_set_high().unwrap();
                channel.test_bus_write(&[3]).unwrap();
                assert_eq!(channel.test_bus_read().unwrap()[0], 4);
                channel.test_output_set_low().unwrap();

                println!("Checked successfully");
            }
        }
    } else {
        println!("No Daughterboard connected");
    }
}
