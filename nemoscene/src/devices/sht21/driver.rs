use embedded_hal::delay::DelayNs;
use rppal::hal::Delay;
use rppal::i2c;

const READ_TEMP_HOLD: u8 = 0xE3;
const READ_HUM_HOLD: u8 = 0xE5;
const READ_TEMP_NO_HOLD: u8 = 0xF3;
const READ_HUM_NO_HOLD: u8 = 0xF5;
const WRITE_REG: u8 = 0xE6;
const READ_REG: u8 = 0xE7;
const RESET: u8 = 0xFE;
const ADDRESS: u16 = 0x40;

pub(crate) struct SHT21Driver;

impl SHT21Driver {
    pub fn read_sensor(command: u8) -> anyhow::Result<u16> {
        let mut data: [u8; 3] = [0; 3];
        let mut ic = i2c::I2c::with_bus(0)?;
        ic.set_slave_address(ADDRESS)?;
        ic.write(&[command])?;
        let mut d = Delay::new();
        d.delay_ms(if command == READ_HUM_HOLD || command == READ_HUM_NO_HOLD {
            30
        } else {
            85
        });
        ic.read(&mut data)?;
        Ok(u16::from_be_bytes([data[0], data[1]]))
    }

    pub fn read_humidity() -> anyhow::Result<f32> {
        let raw = Self::read_sensor(READ_HUM_NO_HOLD)? & !0x0003;
        Ok(-6.0 + (125.0/65536.0) * (raw as f32))
    }

    pub fn read_temperature() -> anyhow::Result<f64> {
        let raw = Self::read_sensor(READ_TEMP_NO_HOLD)?;
        Ok((175.72 * (raw as f64)) / 65536.0 - 46.85)
    }

    pub fn reset() -> anyhow::Result<()> {
        let mut ic = i2c::I2c::new()?;
        ic.set_slave_address(ADDRESS)?;
        ic.write(&[RESET])?;
        let mut d = Delay::new();
        Ok(d.delay_ms(15))
    }
}