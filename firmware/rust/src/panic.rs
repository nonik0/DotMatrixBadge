#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}