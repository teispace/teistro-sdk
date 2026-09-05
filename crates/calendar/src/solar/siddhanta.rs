//! The Surya Siddhanta as a solar model: the text's Sun, and the text's
//! sunrise and sunset in local mean time at the place.

use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place};
use teistro_core::time::LocalMeanTime;
use teistro_siddhanta::SuryaSiddhanta;

use crate::fixed::FixedDay;
use crate::solar::{DayArc, DayLight, SolarModel};

impl SolarModel for SuryaSiddhanta {
    fn sidereal_sun_deg(&self, jd_ut: f64) -> Result<f64, Error> {
        Ok(self.sun_longitude_deg(jd_ut))
    }

    fn day_light(&self, day: FixedDay, place: &Place) -> Result<DayLight, Error> {
        // The text reckons the day in local mean time at the place: its
        // midnight is the civil midnight less the longitude's offset.
        let clock = LocalMeanTime::new(place.longitude);
        let local_midnight =
            JulianDay::try_new(day.jd_at_midnight()?.get() - clock.offset().days())?;
        Ok(
            match SuryaSiddhanta::day_arc(self, local_midnight, place.latitude) {
                Some(arc) => DayLight::Arc(DayArc {
                    sunrise: arc.sunrise.relabel(),
                    sunset: arc.sunset.relabel(),
                }),
                None if self.sun_up_all_day(local_midnight, place.latitude) => DayLight::AlwaysUp,
                None => DayLight::NeverUp,
            },
        )
    }

    fn describe(&self) -> String {
        SuryaSiddhanta::describe(self)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};

    use super::*;
    use crate::CalendarSystem;
    use crate::gregorian::Gregorian;

    #[test]
    fn the_text_answers_as_a_model() {
        let text = SuryaSiddhanta::text();
        let model: &dyn SolarModel = &text;
        assert!(model.describe().starts_with("Surya Siddhanta"));
        let sun = model.sidereal_sun_deg(2_460_413.5).unwrap();
        assert!(sun < 1.0 || sun > 359.0, "{sun}");
        let kathmandu = Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        );
        let day = Gregorian.to_fixed_ymd(2024, 6, 21).unwrap();
        let arc = model.day_arc(day, &kathmandu).unwrap().unwrap();
        // Sunrise near 5:07 local mean time, 23:20 UTC the evening before.
        let midnight_utc = day.jd_at_midnight().unwrap().get();
        let sunrise_utc_hours = (arc.sunrise.get() - midnight_utc) * 24.0;
        assert!((sunrise_utc_hours + 0.6).abs() < 0.1, "{sunrise_utc_hours}");
        assert!(
            (arc.at_fraction(0.5) - f64::midpoint(arc.sunrise.get(), arc.sunset.get())).abs()
                < 1e-12
        );
        let tromso = Place::new(
            Latitude::literal(69.6),
            Longitude::literal(18.9),
            Altitude::literal(0.0),
        );
        assert!(model.day_arc(day, &tromso).unwrap().is_none());
        assert_eq!(model.day_light(day, &tromso).unwrap(), DayLight::AlwaysUp);
        let december = Gregorian.to_fixed_ymd(2024, 12, 21).unwrap();
        assert_eq!(
            model.day_light(december, &tromso).unwrap(),
            DayLight::NeverUp
        );
        assert!(DayLight::AlwaysUp.arc().is_none());
    }
}
