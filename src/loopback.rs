//! The unit tests' loopback HTTP fixtures share this accept.

use std::{
  io,
  net::{TcpListener, TcpStream},
  time::{Duration, Instant},
};

/// Accept the next connection on `listener` by `deadline`, so a fixture server whose
/// client never connects fails instead of hanging. The stream comes back blocking:
/// macOS gives an accepted socket the listener's non-blocking mode, in which a read
/// ignores its timeout and fails at once when the request has not arrived yet.
pub fn accept_by(listener: &TcpListener, deadline: Instant) -> io::Result<TcpStream> {
  listener.set_nonblocking(true)?;
  loop {
    match listener.accept() {
      Ok((stream, _)) => {
        stream.set_nonblocking(false)?;
        return Ok(stream);
      }
      Err(error) if error.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline => {
        std::thread::sleep(Duration::from_millis(10));
      }
      Err(error) => return Err(error),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::{Read, Write};

  #[test]
  fn accepted_streams_wait_for_a_late_request() -> crate::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let client = std::thread::spawn(move || -> io::Result<()> {
      let mut stream = TcpStream::connect(address)?;
      std::thread::sleep(Duration::from_millis(100));
      stream.write_all(b"late")
    });
    let mut stream = accept_by(&listener, Instant::now() + Duration::from_secs(5))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    let mut request = Vec::new();
    let read = stream.read_to_end(&mut request);
    client.join().map_err(|_| "fixture client panicked")??;
    read?;
    assert_eq!(request, b"late");
    Ok(())
  }
}
