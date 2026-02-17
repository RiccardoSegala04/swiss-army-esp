
# Swiss Army Esp
```
                                      .:^
               ^                     /   :
  '`.        /;/                    /    /
  \  \      /;/                    /    /
   \\ \    /;/                    /  ///
    \\ \  /;/                    /  ///
     \  \/_/____________________/    /
      `/                         \  /
      {  -   Swiss Army Esp    -  }'
       \_________________________/
  
```
## Requirements
### Software
- ESP-IDF developement toolkit
- espup: needed to install the required rust compiler toolchain
- cargo
- probe-rs
- telnet: required to connect to Swiss Army Esp CLI
### Hardware
- Recomended: ESP32-S3 devkit-c-1, any other esp32-s3 based board should work with no modifications, other esp32 non -s3 devices should work with changes to Cargo.toml and the cargo config
- SSD1306 based 128x64 i2c oled display
- Infrared transmitter and receiver
- CC1101 sub-GHz radio module
- 4 buttons for navigation

## Project Layout

```
src
├── main.rs
├── devices
│   ├── display
│   │   └── ...
│   └── ...
├── services
│   ├── router.rs  
│   └── ...
└── ui
    ├── app.rs
    ├── elements
    │   ├── ...
    │   ├── top_bar.rs
    │   └── snake
    │       └── ...
    ├── ...
    └── views
        └── ...

```
Ui Views are made of Ui elements and communicate to the peripherals's services via the router

## Building
Once the required software is installed the esp-idf environment must be enabled:
```
source /opt/esp-idf/export.sh   # NOTE this is distribution dependant
```
Then the Firmware can be compiled and uploaded with
```
cargo build --release
cargo run --release
```
This automatically shows debugging output, that can also be seen with
```
probe-rs attach --chip=esp32s3 --preverify --always-print-stacktrace --no-location --catch-hardfault target/xtensa-esp32s3-none-elf/release/swiss-army-esp
```
## User guide
Upon startup the displays shows a menu, the user can navigate and choose a function with the buttons, in the IR and RADIO views signals can be recorded and played back

The device creates a wifi access point, you can connect to it and open the CLI with
```
telnet 192.168.2.1 8080
```

## Links

## Team
- Riccardo Segala
    - UI: 1179
    - CLI: 200 LOC
    - snake: 223 LOC
    - IR driver + service: 171 LOC
- Ettore Beltrame
    - cc1101 driver + service: 929 LOC
    - router: 82 LOC
    - input controller: 69 LOC
