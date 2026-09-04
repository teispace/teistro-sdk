//! Lets the addon link against Node's N-API symbols at load time.
fn main() {
    napi_build::setup();
}
