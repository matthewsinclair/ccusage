fn main() -> std::process::ExitCode {
  match ccusage::cli::run(std::env::args().skip(1).collect()) {
    Ok(()) => std::process::ExitCode::SUCCESS,
    Err(error) => {
      eprintln!("ccusage: {}", ccusage::clean(&error.to_string()));
      std::process::ExitCode::FAILURE
    }
  }
}
