extern crate remote_trait_tests;

#[cfg(all(unix, target_arch = "x86_64"))]
fn main() -> Result<(), String> {
    let args = std::env::args().collect();
    remote_trait_tests::mod_scheduler_main(args);
    Ok(())
}
