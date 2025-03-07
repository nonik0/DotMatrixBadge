use once_cell::unsync::OnceCell;
use random_trait::Random;

// zero-size type in front of the static cell
pub struct Rand;
static RNG_STATE: OnceCell<RngState> = OnceCell::new();

impl Rand {
    pub fn seed(seed: u32) {
        RNG_STATE.set(RngState { value: seed, index: 0 }).expect("RNG already seeded");
    }

    pub fn default() -> &'static RngState {
        RNG_STATE.get().expect("RNG not seeded")
    }
}

#[derive(Clone, Copy, Debug)]
pub struct RngState {
    value: u32,
    index: usize,
}

impl Random for RngState {
    type Error = ();
    fn try_fill_bytes(&mut self, buf: &mut [u8]) -> Result<(), Self::Error> {
        let mut rand_bytes = self.value.to_le_bytes();
        for e in buf.iter_mut() {
            *e = rand_bytes[self.index];
            self.index += 1;

            if self.index == 4 {
                self.value = self.value.wrapping_mul(1664525).wrapping_add(1013904223);
                self.index = 0;
                rand_bytes = self.value.to_le_bytes();
            }
        }
        Ok(())
    }
}
