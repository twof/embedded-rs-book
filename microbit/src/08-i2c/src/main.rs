#![no_main]
#![no_std]

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use core::fmt::Write;
use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate, Measurement};

use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

mod serial_setup;
use core::str::from_utf8;
use heapless::Vec;
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    // A buffer with 32 bytes of capacity
    let mut buffer: Vec<u8, 32> = Vec::new();

    // Code from documentation
    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz1).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    loop {
        buffer.clear();

        while let byte = nb::block!(serial.read()).unwrap() {
            if byte != 13 {
                match buffer.push(byte) {
                    Ok(_) => (),
                    Err(_) => {
                        write!(serial, "Could not push to buffer, likely full.").unwrap();
                        nb::block!(serial.flush()).unwrap();
                    }
                };
                // let letter = byte as char;
                nb::block!(serial.write(byte)).unwrap();
                nb::block!(serial.flush()).unwrap();
            } else {
                nb::block!(serial.write(b'\n')).unwrap();
                nb::block!(serial.write(b'\r')).unwrap();
                nb::block!(serial.flush()).unwrap();
                break;
            }
        }

        let command_string = from_utf8(&buffer).ok().unwrap();

        let command_result: Result<Measurement, &str> = match command_string {
            "magnetometer" => Result::Ok(nb::block!(sensor.mag_data()).unwrap()),
            "accelerometer" => Result::Ok(sensor.accel_data().unwrap()),
            _ => Result::Err("Unknown command"),
        };

        match command_result {
            Ok(data) => write!(
                serial,
                "Measurement: x: {}, y: {}, z: {}",
                data.x, data.y, data.z
            ),
            Err(message) => write!(serial, "{}", message),
        };

        write!(serial, "\n\r");
    }
}
