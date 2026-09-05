//! The time crate's budgets (`docs/03-design/time-and-timezone.md`,
//! section 7): zone resolution, the civil time of an instant, Delta T
//! from the table, TT from UTC, ghati-pala both ways, the local day.

#![allow(
    missing_docs,
    clippy::expect_used,
    reason = "a benchmark binary stops on a bad fixture"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use teistro_calendar::{CalendarDate, Gregorian};
use teistro_core::catalogue::Calendar;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1, Utc};
use teistro_core::settings::PolarDayPolicy;
use teistro_siddhanta::SuryaSiddhanta;
use teistro_time::{
    CivilDateTime, CivilTime, DeltaTModel, EmbeddedTzdb, GhatiPala, Policy, Reckoning, ZoneSpec,
    civil_of, delta_t, ghati_pala, instant_of, local_day, resolve, tt_from_utc, zones,
};

fn benches(c: &mut Criterion) {
    let db = EmbeddedTzdb::shared();
    let civil = CivilDateTime::at(
        CalendarDate::defined(Calendar::Gregorian, 1990, 4, 14),
        CivilTime::new(5, 30, 0).expect("a real time"),
    );
    let kathmandu = ZoneSpec::iana("Asia/Kathmandu");
    let policy = Policy::default();
    c.bench_function("resolve: Asia/Kathmandu through the embedded tzdb", |b| {
        b.iter(|| resolve(black_box(&civil), &kathmandu, &policy, db));
    });
    let lmt = ZoneSpec::local_mean(Longitude::literal(85.324));
    c.bench_function("resolve: local mean time", |b| {
        b.iter(|| resolve(black_box(&civil), &lmt, &policy, db));
    });
    let instant = JulianDay::<Utc>::try_new(2_447_995.489_583_3).expect("finite");
    c.bench_function("civil_of: Asia/Kathmandu", |b| {
        b.iter(|| civil_of(black_box(instant), &kathmandu, db));
    });
    let ut1 = JulianDay::<Ut1>::try_new(2_451_544.5).expect("finite");
    c.bench_function("delta_t: table", |b| {
        b.iter(|| delta_t(black_box(ut1), DeltaTModel::TableThenModel));
    });
    c.bench_function("delta_t: model (1850)", |b| {
        b.iter(|| {
            delta_t(
                black_box(JulianDay::try_new(2_396_758.5).expect("finite")),
                DeltaTModel::TableThenModel,
            )
        });
    });
    c.bench_function("tt_from_utc", |b| {
        b.iter(|| tt_from_utc(black_box(instant), DeltaTModel::TableThenModel));
    });
    let text = SuryaSiddhanta::text();
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let date = CalendarDate::defined(Calendar::Gregorian, 2024, 6, 21);
    c.bench_function("local_day: Kathmandu (Surya Siddhanta arc)", |b| {
        b.iter(|| {
            local_day(
                &text,
                &Gregorian,
                zones::nepal(),
                &place,
                black_box(&date),
                PolarDayPolicy::Undefined,
            )
        });
    });
    let day = local_day(
        &text,
        &Gregorian,
        zones::nepal(),
        &place,
        &date,
        PolarDayPolicy::Undefined,
    )
    .expect("a day");
    let noon = day.sunrise.plus_days(0.25).expect("finite");
    for reckoning in [Reckoning::Civil, Reckoning::Proportional] {
        c.bench_function(&format!("ghati_pala: {reckoning:?}"), |b| {
            b.iter(|| ghati_pala(&day, black_box(noon), reckoning));
        });
        let count = GhatiPala {
            ghati: 15,
            pala: 30,
            vipala: 45,
        };
        c.bench_function(&format!("instant_of: {reckoning:?}"), |b| {
            b.iter(|| instant_of(&day, black_box(count), reckoning));
        });
    }
}

criterion_group!(benches_group, benches);
criterion_main!(benches_group);
