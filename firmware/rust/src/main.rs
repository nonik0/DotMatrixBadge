#![no_std]
#![no_main]

mod panic;

use heapless::Vec;

const NUM_CHARS: usize = 8;
const NUM_ROWS: usize = hcms_29xx::CHAR_HEIGHT;
const NUM_COLS: usize = hcms_29xx::CHAR_WIDTH * NUM_CHARS;
const COLUMN_GAP: usize = 3; // number of "gap" columns between characters

const NUM_SKY_CHARS: usize = 4;
const NUM_SKY_COLS: usize =
    NUM_SKY_CHARS * hcms_29xx::CHAR_WIDTH + (NUM_SKY_CHARS - 1) * COLUMN_GAP;
const SKY_PERIOD: u8 = 12;

const NUM_EARTH_CHARS: usize = 4;
const NUM_EARTH_COLS: usize =
    NUM_EARTH_CHARS * hcms_29xx::CHAR_WIDTH + (NUM_EARTH_CHARS - 1) * COLUMN_GAP;
const EARTH_PERIOD: u8 = 7;

#[arduino_hal::entry]
fn main() -> ! {
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

    let mut sky_cols: Vec<u8, NUM_SKY_COLS> = Vec::new();
    let mut earth_cols: Vec<u8, NUM_EARTH_COLS> = Vec::new();
    let mut sky_count: u8 = 0;
    let mut earth_count: u8 = 0;

    let mut sky_state = SkyState::new();
    let mut earth_state = EarthState::new();

    for _ in 0..NUM_SKY_COLS {
        sky_cols.push(0).unwrap();
    }
    for _ in 0..NUM_EARTH_COLS {
        earth_cols.push(0).unwrap();
    }

    loop {
        sky_count += 1;
        if sky_count >= SKY_PERIOD {
            let new_sky_col = generate_sky_column(&mut sky_state);
            sky_cols.remove(0);
            sky_cols.push(new_sky_col).unwrap();
            sky_count = 0;
        }

        earth_count += 1;
        if earth_count >= EARTH_PERIOD {
            let new_earth_col = generate_earth_column(&mut earth_state);
            earth_cols.remove(0);
            earth_cols.push(new_earth_col).unwrap();
            earth_count = 0;
        }

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
        arduino_hal::delay_ms(15);
    }
}

// col bits: msb+1 is bottom row, lsb is top row

struct SkyState {
    cloud_loc: u8,
    cloud_gap: u8,
    cloud_cur_length: u8,
    cloud_height: u8,
    cloud_length: u8,
    cloud_gap_index: usize,
    cloud_loc_index: usize,
    cloud_height_index: usize,
    cloud_length_index: usize,
}

impl SkyState {
    fn new() -> Self {
        SkyState {
            cloud_loc: 0,
            cloud_gap: 1,
            cloud_cur_length: 0,
            cloud_length: 0,
            cloud_height: 0,
            cloud_gap_index: 0,
            cloud_loc_index: 0,
            cloud_height_index: 0,
            cloud_length_index: 0,
        }
    }
}

fn generate_sky_column(state: &mut SkyState) -> u8 {
    let mut col = 0b1111_1111;

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
        const LOC_PATTERN: [u8; 11] = [1, 4, 3, 2, 1, 4, 3, 4, 2, 3, 1];
        const HEIGHT_PATTERN: [u8; 5] = [2, 2, 3, 2, 3];
        const LENGTH_PATTERN: [u8; 7] = [7, 12, 8, 10, 8, 13, 11];
        const GAP_PATTERN: [u8; 3] = [3, 6, 9];
        state.cloud_cur_length = 0;
        state.cloud_gap_index = (state.cloud_gap_index + 1) % GAP_PATTERN.len();
        state.cloud_loc_index = (state.cloud_loc_index + 1) % LOC_PATTERN.len();
        state.cloud_height_index = (state.cloud_height_index + 1) % HEIGHT_PATTERN.len();
        state.cloud_length_index = (state.cloud_length_index + 1) % LENGTH_PATTERN.len();
        state.cloud_gap = GAP_PATTERN[state.cloud_gap_index];
        state.cloud_loc = LOC_PATTERN[state.cloud_loc_index];
        state.cloud_height = HEIGHT_PATTERN[state.cloud_height_index];
        state.cloud_length = LENGTH_PATTERN[state.cloud_length_index];
        if state.cloud_height == 3 && state.cloud_loc == 4 {
            state.cloud_loc = 3;
        }
    }

    col
}

struct EarthState {
    mountain_cur_height: u8,
    mountain_height: u8,
    mountain_increment: i8,
    mountain_index: usize,
}

impl EarthState {
    fn new() -> Self {
        EarthState {
            mountain_index: 0,
            mountain_cur_height: 0,
            mountain_height: 7,
            mountain_increment: 1,
        }
    }
}

fn generate_earth_column(state: &mut EarthState) -> u8 {
    // actually drawing "sky" behind mountains
    let mut col = 0b0000_0000;
    for _ in 0..(hcms_29xx::CHAR_HEIGHT as u8 - state.mountain_cur_height) {
        col = col << 1 | 1;
    }

    state.mountain_cur_height = (state.mountain_cur_height as i8 + state.mountain_increment) as u8;

    // start new mountain
    if state.mountain_cur_height == 0 && state.mountain_increment < 0 {
        const HEIGHT_PATTERN: [u8; 7] = [7, 5, 6, 6, 7, 4, 6];
        state.mountain_index = (state.mountain_index + 1) % HEIGHT_PATTERN.len();
        state.mountain_height = HEIGHT_PATTERN[state.mountain_index];
        state.mountain_increment *= -1;
    } else if state.mountain_cur_height == state.mountain_height {
        state.mountain_increment *= -1;
    }

    col
}
