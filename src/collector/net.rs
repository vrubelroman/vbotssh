use anyhow::{Context, Result};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NetworkCounters {
    pub receive_bytes: u64,
    pub transmit_bytes: u64,
}

pub fn collect_local_network_counters() -> Result<NetworkCounters> {
    let payload = std::fs::read_to_string("/proc/net/dev").context("failed to read /proc/net/dev")?;
    parse_proc_net_dev(&payload)
}

pub fn parse_proc_net_dev(payload: &str) -> Result<NetworkCounters> {
    let mut counters = NetworkCounters::default();

    for line in payload.lines().skip(2) {
        let Some((interface, stats)) = line.split_once(':') else {
            continue;
        };

        let interface = interface.trim();
        if interface.is_empty() || interface == "lo" {
            continue;
        }

        let fields = stats.split_whitespace().collect::<Vec<_>>();
        if fields.len() < 16 {
            continue;
        }

        let receive_bytes = fields[0]
            .parse::<u64>()
            .with_context(|| format!("failed to parse rx bytes for interface {interface}"))?;
        let transmit_bytes = fields[8]
            .parse::<u64>()
            .with_context(|| format!("failed to parse tx bytes for interface {interface}"))?;

        counters.receive_bytes = counters.receive_bytes.saturating_add(receive_bytes);
        counters.transmit_bytes = counters.transmit_bytes.saturating_add(transmit_bytes);
    }

    Ok(counters)
}

#[cfg(test)]
mod tests {
    use super::parse_proc_net_dev;

    #[test]
    fn sums_non_loopback_interfaces() {
        let payload = r#"Inter-|   Receive                                                |  Transmit
 face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed
    lo: 100 0 0 0 0 0 0 0 200 0 0 0 0 0 0 0
  eth0: 1000 0 0 0 0 0 0 0 2000 0 0 0 0 0 0 0
 wlan0: 3000 0 0 0 0 0 0 0 4000 0 0 0 0 0 0 0
"#;

        let counters = parse_proc_net_dev(payload).unwrap();
        assert_eq!(counters.receive_bytes, 4000);
        assert_eq!(counters.transmit_bytes, 6000);
    }
}
