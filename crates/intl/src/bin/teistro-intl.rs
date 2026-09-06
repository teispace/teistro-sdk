//! The `teistro-intl` command line: every command is a library function
//! in `teistro_intl::cli`, this file the shell.
fn main() -> std::process::ExitCode {
    teistro_intl::cli::main()
}
