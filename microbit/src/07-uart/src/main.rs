#![no_main]
#![no_std]

use core::{fmt::Write, iter::FromIterator};
use cortex_m_rt::entry;
use heapless::Vec;
use panic_rtt_target as _;
use rtt_target::rtt_init_print;

use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

mod serial_setup;
use heapless::String;
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

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
                break;
            }
        }
        // TODO Receive a user request. Each user request ends with ENTER
        // NOTE `buffer.push` returns a `Result`. Handle the error by responding
        // with an error message.

        // TODO Send back the reversed string
        let reverse_vec: Vec<&u8, 32> = Vec::from_iter(buffer.iter().rev());
        let reverse_char_vec: Vec<char, 32> = reverse_vec
            .into_iter()
            .map(|element| *element as char)
            .collect();

        let reverse_string: String<32> = reverse_char_vec.into_iter().collect();

        nb::block!(match serial.write_str(&reverse_string) {
            Ok(_) => nb::Result::Ok(()),
            Err(err) => nb::Result::Err(nb::Error::Other(err)),
        })
        .unwrap();
        nb::block!(serial.flush()).unwrap();
        // nb::block!(foo).unwrap();
    }
}
