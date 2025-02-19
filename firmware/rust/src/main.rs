#![no_std]
#![no_main]

// TODO: implement watchdog?

use panic_halt as _;

const NUM_CHARS: usize = 8;
const MESSAGE: &[u8] = b"Stella and Beau and Stevie and Louie and ";

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led_pin = pins.d13.into_output();

    // high impedance pins
    pins.sck.into_floating_input();
    pins.mosi.into_floating_input();
    pins.d9.into_floating_input();
    pins.d5.into_floating_input();

    let mut display = hcms_29xx::Hcms29xx::new(
        NUM_CHARS,                                // Number of characters in the display
        pins.d0.into_output().downgrade(),        // Data pin
        pins.d1.into_output().downgrade(),        // Clock pin
        pins.d11.into_output().downgrade(),       // Chip select pin
        pins.d2.into_output().downgrade(),        // Reset pin
        Some(pins.d3.into_output().downgrade()),  // Optional: Enable pin
        Some(pins.d6.into_output().downgrade()),  // Optional: Write pin
        Some(pins.d10.into_output().downgrade()), // Optional: Read pin
    )
    .unwrap();
    display.begin().unwrap();
    display.display_unblank().unwrap();
    display.set_current(1).unwrap();
    //display.set_int_osc().unwrap(); now default

    // test max current/power draw
    // display.set_current(crate::hcms29xx::constants::control_word_0::current::MAX_12_8MA).unwrap();
    // display.set_brightness(crate::hcms29xx::constants::control_word_0::MAX_BRIGHTNESS).unwrap();

    let mut cursor: usize = 0;
    let mut count: u16 = 0;
    let mut buf: [u8; NUM_CHARS] = [0; NUM_CHARS];
    loop {
        count = (count + 1) % 10000;
        if (count % 30) == 0 {
            cursor = (cursor + 1) % MESSAGE.len();
        }
        if (count % 500) == 0 {
            led_pin.toggle();
        }

        for i in 0..4 {
            let index = (cursor + i as usize) % MESSAGE.len();
            buf[i as usize] = MESSAGE[index];
        }

        let mut count_dec = count;
        for i in (0..4).rev() {
            buf[i as usize + 4] = if count_dec > 0 {
                (count_dec % 10) as u8 + b'0'
            } else {
                b' '
            };
            count_dec /= 10;
        }

        display.print_c_string(&buf).unwrap();
        arduino_hal::delay_ms(1);
    }
}
