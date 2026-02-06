use embedded_hal::digital::InputPin;
use embedded_hal::pwm::SetDutyCycle;

struct Controller<PWMpin, InPin> {
    rgb_led: (PWMpin, PWMpin, PWMpin),
    confirm_button: InPin,
}

impl<PWMpin, InPin> Controller<PWMpin, InPin>
where
    PWMpin: SetDutyCycle,
    InPin: InputPin,
{
    pub fn new(rgb_led: (PWMpin, PWMpin, PWMpin), confirm_button: InPin) -> Self {
        Self {
            rgb_led: rgb_led,
            confirm_button: confirm_button,
        }
    }

    pub fn set_led(&mut self, red: u8, green: u8, blue: u8) -> Result<(), PWMpin::Error> {
        self.rgb_led.0.set_duty_cycle_percent(red)?;
        self.rgb_led.1.set_duty_cycle_percent(green)?;
        self.rgb_led.2.set_duty_cycle_percent(blue)?;
        Ok(())
    }

    pub fn is_confirm_pressed(&mut self) -> Result<bool, InPin::Error> {
        self.confirm_button.is_low()
    }
}
