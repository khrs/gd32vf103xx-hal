//! Timers

use crate::time::Hertz;
use crate::rcu::{Rcu, BaseFrequency, Enable, Reset};

use gd32vf103_pac::{TIMER0, TIMER1, TIMER2, TIMER3, TIMER4, TIMER5, TIMER6};
use embedded_hal::timer::{CountDown, Periodic};
use void::Void;

/// Hardware timer
pub struct Timer<TIM> {
    pub(crate) tim: TIM,
    pub(crate) timer_clock: Hertz,
    pub(crate) timeout: Hertz,
}

/// Interrupt events
pub enum Event {
    /// Timer timed out / count down ended
    TimeOut,
}

macro_rules! hal {
    ($($TIM:ident: $tim:ident,)+) => {
        $(
            impl Timer<$TIM> {
                pub fn $tim<T>(timer: $TIM, timeout: T, rcu: &mut Rcu) -> Self
                    where T: Into<Hertz> {
                    $TIM::enable(rcu);
                    $TIM::reset(rcu);
                    let mut t = Timer {
                        timer_clock: $TIM::base_frequency(&rcu.clocks),
                        tim: timer,
                        timeout: Hertz(0),
                    };
                    t.start(timeout);

                    t
                }

                /// Releases the TIMER peripheral
                pub fn free(self) -> $TIM {
                    self.tim.ctl0.modify(|_, w| w.cen().clear_bit());
                    self.tim
                }
            }

            impl Periodic for Timer<$TIM> {}

            impl CountDown for Timer<$TIM> {
                type Time = Hertz;

                fn start<T>(&mut self, timeout: T)
                    where
                        T: Into<Hertz>,
                {
                    self.timeout = timeout.into();

                    self.tim.ctl0.modify(|_, w| w.cen().clear_bit());
                    self.tim.cnt.reset();

                    let ticks = self.timer_clock.0 / self.timeout.0;
                    let psc = ((ticks - 1) / (1 << 16)) as u16;
                    let car = (ticks / ((psc + 1) as u32)) as u16;
                    self.tim.psc.write(|w| unsafe { w.bits(psc) } );
                    self.tim.car.write(|w| unsafe { w.bits(car) } );
                    self.tim.ctl0.write(|w| { w
                        .updis().clear_bit()
                        .cen().set_bit()
                    });
                }

                fn wait(&mut self) -> nb::Result<(), Void> {
                    if self.tim.intf.read().upif().bit_is_clear() {
                        Err(nb::Error::WouldBlock)
                    } else {
                        self.tim.intf.modify(|_r, w| w.upif().clear_bit());
                        Ok(())
                    }
                }
            }
        )+
    }
}

hal! {
    TIMER0: timer0,
    TIMER1: timer1,
    TIMER2: timer2,
    TIMER3: timer3,
    TIMER4: timer4,
    TIMER5: timer5,
    TIMER6: timer6,
}
