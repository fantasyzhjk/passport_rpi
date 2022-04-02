use std::error::Error;
use rppal::gpio::Gpio;

const GPIO_LED_R: u8 = 23;
const GPIO_LED_G: u8 = 24;

pub enum LEDState {
    GREEN,
    RED,
    OFF
}

pub fn toggle_light(state: LEDState) -> Result<(), Box<dyn Error>> {
    let mut pin_r = Gpio::new()?.get(GPIO_LED_R)?.into_output();
    let mut pin_g = Gpio::new()?.get(GPIO_LED_G)?.into_output();
    match state {
        LEDState::GREEN => {
            pin_g.set_high();
            pin_r.set_low();
        }
        LEDState::RED => {
            pin_r.set_high();
            pin_g.set_low();
        }
        LEDState::OFF => {
            pin_g.set_low();
            pin_r.set_low();
        }
    }
    Ok(())
}