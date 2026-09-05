use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};

fn main() {
    let latitude = Latitude::try_new(27.7).unwrap();
    let longitude = Longitude::try_new(85.3).unwrap();
    let altitude = Altitude::try_new(1400.0).unwrap();
    // Latitude and longitude swapped: a compile error, never a wrong chart.
    let _ = Place::new(longitude, latitude, altitude);
}
