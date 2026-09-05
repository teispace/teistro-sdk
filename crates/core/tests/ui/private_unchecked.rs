use teistro_core::quantity::HouseNumber;

fn main() {
    // The unchecked constructor is not public.
    let _ = HouseNumber::new_unchecked(13);
}
