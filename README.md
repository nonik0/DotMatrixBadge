# Nick's Original Name Indicator Knicknack (version 0)

## Overview
It's a knicknack to indicate your name with the original design by me, Nick. Version 0. NONIK0 for short! The only version there will ever be. I wanted a name badge to wear sometimes and most other times just use for a fancy LED decoration. Add in a tad of scope creep and you have also have some games and beeps, too.

### Hardware
The hardware is designed in KiCad 8.0 and all design files are included in the [hardware/kicad8.0](hardware/kicad8.0) directory.

The main hardware components are:
- [HCMS-2914](https://www.broadcom.com/products/leds-and-displays/smart-alphanumeric-displays/serial-interface/hcms-2914) 5x7 dot matrix character display
- [ATtiny1604](https://www.microchip.com/en-us/product/attiny1604) 8-bit MCU
- 2x buttons

### Firmware

The firmware is written in Rust and can be found in the [firmware/rust](firmware/rust) directory. As part of the work, I wrote a new [HCMS-29xx driver](https://github.com/nonik0/TODO) (TODO: split out from firmware) and forked avr-hal to add a Feather32u4 support, which I used for bootstrapping the HCMS-29xx driver work. There is also an older PlatformIO project targetting the Adafruit Feather in the [firmware/cpp](firmware/cpp) directory. TODO: pictures of prototype device

## Design Images

<p align="center" width="100%">
  <img src="https://github.com/nonik0/NONIK0/blob/main/hardware/images/schematic.png" />
  <img src="https://github.com/nonik0/NONIK0/blob/main/hardware/images/pcb_layout.png"/>
  <img src="https://github.com/nonik0/NONIK0/blob/main/hardware/images/render_front.png" />
  <img src="https://github.com/nonik0/NONIK0/blob/main/hardware/images/render_back.png" />
</p>

## Work Left
- order boards
- assemble
- try out some games and stuff

