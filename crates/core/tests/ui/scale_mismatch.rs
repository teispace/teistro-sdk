use teistro_core::quantity::{JulianDay, Tt, Ut1};

fn needs_tt(_: JulianDay<Tt>) {}

fn main() {
    let ut1 = JulianDay::<Ut1>::J2000;
    // A UT1 instant where TT is expected: a compile error.
    needs_tt(ut1);
}
