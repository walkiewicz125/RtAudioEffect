use std::{borrow::Borrow, marker::PhantomData};

use esp_idf_hal::{
    gpio::OutputPin,
    ledc::{LedcChannel, LedcDriver, LedcTimer, LedcTimerDriver},
    peripheral::Peripheral,
};
use esp_idf_sys::EspError;

pub struct Servo<'d> {
    driver: LedcDriver<'d>,
    min_limit: f32,
    max_limit: f32,
    _p: PhantomData<&'d mut ()>,
}

impl<'d> Servo<'d> {
    pub fn new<Channel, Timer, TimerDriver>(
        timer_driver: TimerDriver,
        channel: impl Peripheral<P = Channel> + 'd,
        pin: impl Peripheral<P = impl OutputPin> + 'd,
    ) -> Result<Self, EspError>
    where
        Channel: LedcChannel<SpeedMode = <Timer as LedcTimer>::SpeedMode>,
        Timer: LedcTimer + 'd,
        TimerDriver: Borrow<LedcTimerDriver<'d, Timer>>,
    {
        let driver = LedcDriver::new(channel, timer_driver, pin)?;
        let max_duty = driver.get_max_duty();
        let min_limit = max_duty as f32 * 0.025; // 25ms
        let max_limit = max_duty as f32 * 0.125; // 125ms
        Ok(Self {
            driver,
            min_limit,
            max_limit,
            _p: PhantomData,
        })
    }

    pub fn set_duty(&mut self, set_point: f32) -> Result<(), EspError> {
        self.driver.set_duty(self.map(set_point))
    }

    fn map(&self, set_point: f32) -> u32 {
        let set_point = set_point.max(-1.0).min(1.0); // clamp
        let set_point = (set_point + 1.0) / 2.0; // normalize
        let range = self.max_limit - self.min_limit; // calculate range
        (self.min_limit as f32 + range as f32 * set_point) as u32 //
    }
}
