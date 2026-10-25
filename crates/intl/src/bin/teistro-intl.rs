//! The `teistro-intl` command line: every command is a library function
//! in `teistro_intl::cli`, this file the shell.

#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

fn main() -> std::process::ExitCode {
    teistro_intl::cli::main()
}
