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
const SKY_PERIOD: u8 = 4;

const NUM_EARTH_CHARS: usize = 4;
const NUM_EARTH_COLS: usize =
    NUM_EARTH_CHARS * hcms_29xx::CHAR_WIDTH + (NUM_EARTH_CHARS - 1) * COLUMN_GAP;
const EARTH_PERIOD: u8 = 3;

#[arduino_hal::entry]
fn main() -> ! {
    Rand::seed(12345); // Initialize the RNG with a seed

    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    //let mut led_pin = pins.d13.into_output();

    // high impedance pins
    pins.sck.into_floating_input();
    pins.mosi.into_floating_input();
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

    let mut sky_state = SkyState::new();
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

struct SkyState {
    cloud_loc: u8,
    cloud_gap: u8,
    cloud_cur_length: u8,
    cloud_height: u8,
    cloud_length: u8,
}

impl SkyState {
    fn new() -> Self {
        SkyState {
            cloud_loc: 0,
            cloud_gap: 1,
            cloud_cur_length: 0,
            cloud_length: 0,
            cloud_height: 0,
        }
    }

    fn update_sky(&mut self) {
        let mut rng = Rand::default();
        self.cloud_gap = rng.get_u8() % 10 + 1;
        self.cloud_loc = rng.get_u8() % (NUM_ROWS as u8 - 2);
        self.cloud_height = rng.get_u8() % 3 + 2;
        self.cloud_length = rng.get_u8() % 10 + 5;
    }
}

fn generate_sky_column(state: &mut SkyState) -> u8 {
    // clouds are drawn as silhouettes, i.e. sky is on and cloud is off
    let mut col = 0b0111_1111;

    if state.cloud_gap > 0 {
        state.cloud_gap -= 1;
    } else if state.cloud_cur_length < state.cloud_length {
        for i in 0..NUM_ROWS {
            let bit = if (i as u8) >= state.cloud_loc
                && (i as u8) < state.cloud_loc + state.cloud_height
            {
                0
            } else {
                1
            };
            col = col << 1 | bit;
        }
        state.cloud_cur_length += 1;
    } else {
        state.cloud_cur_length = 0;
        state.update_sky();
    }

    col
}

struct MountainState {
    mountain_cur_height: u8,
    mountain_height: u8,
    mountain_increment: i8,
}

impl MountainState {
    fn new() -> Self {
        MountainState {
            mountain_cur_height: 0,
            mountain_height: 7,
            mountain_increment: 1,
        }
    }

    fn update_mountain(&mut self) {
        let mut rng = Rand::default();
        self.mountain_height = rng.get_u8() % 4 + 4;
    }
}

fn generate_mountain_column(state: &mut MountainState) -> u8 {
    // mountains are drawn as silhouettes, i.e. sky is on and mountain is off
    let mut col = 0b0000_0000;
    for _ in 0..(hcms_29xx::CHAR_HEIGHT as u8 - state.mountain_cur_height) {
        col = col << 1 | 1;
    }

    state.mountain_cur_height = (state.mountain_cur_height as i8 + state.mountain_increment) as u8;

    // start new mountain
    if state.mountain_cur_height == 0 && state.mountain_increment < 0 {
        state.update_mountain();
        state.mountain_increment *= -1;
    } else if state.mountain_cur_height == state.mountain_height {
        state.mountain_increment *= -1;
    }

    col
}
