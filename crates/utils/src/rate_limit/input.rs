use crate::rate_limit::ActionType;
use std::{
  future::Ready,
  net::{IpAddr, Ipv4Addr, SocketAddr},
  str::FromStr,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LemmyInput(pub(crate) RateLimitIpAddr, pub(crate) ActionType);

pub(crate) type LemmyInputFuture = Ready<Result<LemmyInput, actix_web::Error>>;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) enum RateLimitIpAddr {
  V4(Ipv4Addr),
  V6([u16; 4]),
}

#[expect(clippy::expect_used)]
impl From<IpAddr> for RateLimitIpAddr {
  fn from(value: IpAddr) -> Self {
    match value {
      IpAddr::V4(addr) => RateLimitIpAddr::V4(addr),
      IpAddr::V6(addr) => RateLimitIpAddr::V6(
        addr.segments()[..4]
          .try_into()
          .expect("byte array is correct length"),
      ),
    }
  }
}

/// Generate a raw byte key for backend which uses less memory.
pub(crate) fn raw_ip_key(ip_str: Option<&str>) -> RateLimitIpAddr {
  parse_ip(ip_str).into()
}

fn parse_ip(addr: Option<&str>) -> IpAddr {
  if let Some(addr) = addr {
    if let Ok(ip) = IpAddr::from_str(addr) {
      return ip;
    } else if let Ok(socket) = SocketAddr::from_str(addr) {
      return socket.ip();
    }
  }
  Ipv4Addr::new(127, 0, 0, 1).into()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::error::LemmyResult;

  #[test]
  fn test_get_ip() -> LemmyResult<()> {
    // Check that IPv4 addresses are preserved
    assert_eq!(
      raw_ip_key(Some("142.250.187.206")),
      "142.250.187.206".parse::<IpAddr>()?.into()
    );
    // Check that IPv6 addresses are grouped into /64 subnets
    assert_eq!(
      raw_ip_key(Some("2a00:1450:4009:81f::200e")),
      RateLimitIpAddr::V6([0x2a00, 0x1450, 0x4009, 0x81f])
    );
    assert_eq!(
      raw_ip_key(Some("[2a00:1450:4009:81f::200e]:123")),
      RateLimitIpAddr::V6([0x2a00, 0x1450, 0x4009, 0x81f])
    );
    Ok(())
  }
}
