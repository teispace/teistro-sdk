use teistro_core::angle::Nas;

fn main() {
    // A bare f64 is not an angle; the only way in is through `Degrees`.
    let _: Nas = Nas::from_degrees(222.5763);
}
