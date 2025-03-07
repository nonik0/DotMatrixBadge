#![no_std]
#![no_main]

mod panic;
mod random;

use heapless::Vec;
use random::Rand;
use random_trait::Random;

const NUM_CHARS: usize = 8;
const NUM_ROWS: usize = hcms_29xx::CHAR_HEIGHT;
const NUM_COLS: usize = hcms_29xx::CHAR_WIDTH * NUM_CHARS;
const COLUMN_GAP: usize = 2; // number of "gap" columns between characters
const BASE_DELAY_MS: u16 = 100;

const NUM_SKY_CHARS: usize = 4;
const NUM_SKY_COLS: usize =
    NUM_SKY_CHARS * hcms_29xx::CHAR_WIDTH + (NUM_SKY_CHARS - 1) * COLUMN_GAP;
const SKY_PERIOD: u8 = 7;

const NUM_EARTH_CHARS: usize = 4;
const NUM_EARTH_COLS: usize =
    NUM_EARTH_CHARS * hcms_29xx::CHAR_WIDTH + (NUM_EARTH_CHARS - 1) * COLUMN_GAP;
const EARTH_PERIOD: u8 = 3;

#[arduino_hal::entry]
fn main() -> ! {
    Rand::seed(12345); // Initialize the RNG with a seed

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());

    // read voltage from floating pin for reasonable entropy
    let entropy_pin = pins.a0.into_analog_input(&mut adc);
    let seed_value_1 = entropy_pin.analog_read(&mut adc);
    let seed_value_2 = entropy_pin.analog_read(&mut adc);
    let seed_value = (seed_value_1 as u32) << 16 | seed_value_2 as u32;
    Rand::seed(seed_value);

    // high impedance pins
    //pins.sck.into_floating_input();
    //pins.mosi.into_floating_input();
    //pins.sck.into_floating_input();
    //pins.mosi.into_floating_input();    
    pins.d9.into_floating_input();
    pins.d5.into_floating_input();

    let mut display = hcms_29xx::Hcms29xx::<NUM_CHARS, _, _, _, _, _, _, _>::new(
        pins.d0.into_output(),  // Data pin
        pins.d1.into_output(),  // RS pin
        pins.d11.into_output(), // Clock pin
        pins.d2.into_output(),  // CE pin
        pins.d3.into_output(),  // Optional: Blank pin
        pins.d6.into_output(),  // Optional: OscSel pin
        pins.d10.into_output(), // Optional: Reset pin
    )
    .unwrap();

    display.begin().unwrap();
    display.display_unblank().unwrap();
    display
        .set_peak_current(hcms_29xx::PeakCurrent::Max6_4Ma)
        .unwrap();

    // col bits: msb+1 is bottom row, lsb is top row, i.e. 0b0111_1111 is all on
    let mut sky_cols: Vec<u8, NUM_SKY_COLS> = Vec::new();
    let mut earth_cols: Vec<u8, NUM_EARTH_COLS> = Vec::new();
    let mut sky_count: u8 = 0;
    let mut earth_count: u8 = 0;

    let mut sky_state = CloudState::new();
    let mut earth_state = MountainState::new();

    // fill display buffer with initial columns
    for _ in 0..NUM_SKY_COLS {
        let new_sky_col = generate_sky_column(&mut sky_state);
        sky_cols.push(new_sky_col).unwrap();
    }
    for _ in 0..NUM_EARTH_COLS {
        let new_earth_col = generate_mountain_column(&mut earth_state);
        earth_cols.push(new_earth_col).unwrap();
    }

    // sky and mountains will update at different rates for parallax effect (embassy would be nice)
    loop {
        sky_count = (sky_count + 1) % SKY_PERIOD;
        if sky_count == 0 {
            let new_sky_col = generate_sky_column(&mut sky_state);
            sky_cols.remove(0);
            sky_cols.push(new_sky_col).unwrap();
        }

        earth_count = (earth_count + 1) % EARTH_PERIOD;
        if earth_count == 0 {
            let new_earth_col = generate_mountain_column(&mut earth_state);
            earth_cols.remove(0);
            earth_cols.push(new_earth_col).unwrap();
        }

        // TODO: can overlay both to display on single row of characters
        let mut cols: Vec<u8, NUM_COLS> = Vec::new();
        for (i, &col) in sky_cols.iter().enumerate() {
            if i % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                cols.push(col).unwrap();
            }
        }
        for (i, &col) in earth_cols.iter().enumerate() {
            if i % (hcms_29xx::CHAR_WIDTH + COLUMN_GAP) < hcms_29xx::CHAR_WIDTH {
                cols.push(col).unwrap();
            }
        }

        display.print_cols(&cols).unwrap();
        arduino_hal::delay_ms(BASE_DELAY_MS);
    }
}

struct CloudState {
    loc: u8,
    gap: u8,
    cur_length: u8,
    height: u8,
    length: u8,
}

impl CloudState {
    fn new() -> Self {
        CloudState {
            loc: 0,
            gap: 1,
            cur_length: 0,
            length: 0,
            height: 0,
        }
    }

    fn next_cloud(&mut self) {
        let mut rng = Rand::default();
        self.gap = rng.get_u8() % 10 + 1;
        self.loc = 1 + rng.get_u8() % (NUM_ROWS as u8 - 2);
        self.height = rng.get_u8() % 3 + 2;
        self.length = rng.get_u8() % 10 + 5;
    }
}

fn generate_sky_column(state: &mut CloudState) -> u8 {
    // clouds are drawn as silhouettes, i.e. sky is on and cloud is off
    let mut col = 0b0111_1111;

    if state.gap > 0 {
        state.gap -= 1;
    } else if state.cur_length < state.length {
        for i in 0..NUM_ROWS {
            let bit = if (i as u8) >= state.loc
                && (i as u8) < state.loc + state.height
            {
                0
            } else {
                1
            };
            col = col << 1 | bit;
        }
        state.cur_length += 1;
    } else {
        state.cur_length = 0;
        state.next_cloud();
    }

    col
}

struct MountainState {
    cur_height: u8,
    cur_length: u8,
    height: u8,
    length: u8,
    increment: i8,
}

impl MountainState {
    fn new() -> Self {
        MountainState {
            cur_height: 0,
            cur_length: 0,
            height: 7,
            length: 15,
            increment: 1,
        }
    }

    fn next_mountain(&mut self) {
        let mut rng = Rand::default();
        self.height = rng.get_u8() % 4 + 4;
        self.increment = 1;
    }

    // fn next_mountain(&mut self) {
    //     //const MIN_HEIGHT: u8 = 3;
    //     //const MAX_HEIGHT: u8 = 7;
    //     //let mut rng = Rand::default();
    //     // self.cur_height remains the same
    //     self.cur_length = self.cur_height;
    //     //self.height = MIN_HEIGHT + rng.get_u8() % (MAX_HEIGHT - MIN_HEIGHT);
    //     self.height = 5; //self.cur_height + rng.get_u8() % (MAX_HEIGHT - self.cur_height + 1); // mountain needs to match current height at least
    //     self.length = 7; //self.height + 1 + rng.get_u8() % (self.height - 1); // length range: [height + 1, 2*height-1]
    //     self.increment = 1;
    // }
}

fn generate_mountain_column(state: &mut MountainState) -> u8 {
    // mountains are drawn as silhouettes, i.e. sky is on and mountain is off
    let mut col = 0b0000_0000;
    for _ in 0..(hcms_29xx::CHAR_HEIGHT as u8 - state.cur_height) {
        col = col << 1 | 1;
    }

    state.cur_height = (state.cur_height as i8 + state.increment) as u8;
    state.cur_length += 1;

    // start new mountain
    if state.cur_height == 0 && state.increment < 0 {
    //if state.cur_length >= state.length {
        state.next_mountain();
    } else if state.cur_height >= state.height {
        state.increment *= -1;
    }

    col
}
