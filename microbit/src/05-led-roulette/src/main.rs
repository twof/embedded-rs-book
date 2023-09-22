#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::rtt_init_print;
use panic_rtt_target as _;
use microbit::{
    board::Board,
    display::blocking::Display,
    hal::Timer,
};
use core::cmp::max;
use core::cmp::min;
use core::convert::TryFrom;

#[entry]
fn main() -> ! {
    rtt_init_print!();

    let board = Board::take().unwrap();
    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);
    let mut display_layout = [
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0],
    ];

    loop {
        for i in 0..16  {
            let pos = calculate_position(i);
            display_layout[pos.1][pos.0] = 1;
            display.show(&mut timer, display_layout, 10);
            display_layout[pos.1][pos.0] = 0;
        }
    }
}

fn calculate_position(i: i32) -> (usize, usize) {
    let y = usize::try_from(min(16 - i, min(4, max(i - 4, 0)))).unwrap();
    let x = usize::try_from(max(0, min(12 - i, min(i, 4)))).unwrap();
    (x, y)
}