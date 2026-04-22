use std::process::Command;

use anyhow::{Context, Result};

use crate::model::DockerContainerInfo;

pub struct DockerSnapshot {
    pub containers: Vec<DockerContainerInfo>,
    pub error: Option<String>,
}

pub fn collect_local_docker_snapshot() -> Result<DockerSnapshot> {
    let output = Command::new("docker")
        .args([
            "ps",
            "--format",
            "{{.Image}}\t{{.RunningFor}}\t{{.Status}}",
        ])
        .output()
        .context("failed to run docker ps")?;

    if output.status.success() {
        Ok(DockerSnapshot {
            containers: parse_docker_ps_output(&String::from_utf8_lossy(&output.stdout)),
            error: None,
        })
    } else {
        Ok(DockerSnapshot {
            containers: Vec::new(),
            error: Some(compact_error(&String::from_utf8_lossy(&output.stderr), "docker ps failed")),
        })
    }
}

pub fn parse_docker_ps_output(stdout: &str) -> Vec<DockerContainerInfo> {
    stdout
        .lines()
        .filter_map(|line| {
            let line = line.trim_end();
            if line.is_empty() {
                return None;
            }

            let mut parts = line.splitn(3, '\t');
            let image = parts.next()?.trim().to_string();
            let created = parts.next().unwrap_or_default().trim().to_string();
            let status = parts.next().unwrap_or_default().trim().to_string();

            Some(DockerContainerInfo {
                image,
                created,
                status,
            })
        })
        .collect()
}

fn compact_error(stderr: &str, fallback: &str) -> String {
    stderr
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
        .unwrap_or_else(|| fallback.to_string())
}

#[cfg(test)]
mod tests {
    use super::parse_docker_ps_output;

    #[test]
    fn parses_docker_ps_lines() {
        let payload = "\
nginx:latest\t2 hours ago\tUp 2 hours
postgres:16\t3 days ago\tUp 3 days (healthy)
";

        let containers = parse_docker_ps_output(payload);
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[0].image, "nginx:latest");
        assert_eq!(containers[1].status, "Up 3 days (healthy)");
    }
}
