use anyhow::Result;
use bollard::container::{
    ListContainersOptions, RemoveContainerOptions, RestartContainerOptions,
    StartContainerOptions, StopContainerOptions,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::DockerClient;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Running,
    Paused,
    Restarting,
    Exited,
    Dead,
    Created,
    Removing,
    Unknown,
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContainerState::Running => write!(f, "Running"),
            ContainerState::Paused => write!(f, "Paused"),
            ContainerState::Restarting => write!(f, "Restarting"),
            ContainerState::Exited => write!(f, "Exited"),
            ContainerState::Dead => write!(f, "Dead"),
            ContainerState::Created => write!(f, "Created"),
            ContainerState::Removing => write!(f, "Removing"),
            ContainerState::Unknown => write!(f, "Unknown"),
        }
    }
}

impl ContainerState {
    pub fn is_running(&self) -> bool {
        matches!(self, ContainerState::Running)
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" => ContainerState::Running,
            "paused" => ContainerState::Paused,
            "restarting" => ContainerState::Restarting,
            "exited" => ContainerState::Exited,
            "dead" => ContainerState::Dead,
            "created" => ContainerState::Created,
            "removing" => ContainerState::Removing,
            _ => ContainerState::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub private_port: u16,
    pub public_port: Option<u16>,
    pub protocol: String,
    pub ip: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub image_id: String,
    pub state: ContainerState,
    pub status: String,
    pub created: Option<DateTime<Utc>>,
    pub ports: Vec<PortMapping>,
    pub labels: HashMap<String, String>,
    pub command: Option<String>,
    pub size_rw: Option<i64>,
    pub size_root_fs: Option<i64>,
}

impl ContainerInfo {
    pub fn short_id(&self) -> &str {
        if self.id.len() >= 12 {
            &self.id[..12]
        } else {
            &self.id
        }
    }

    pub fn display_ports(&self) -> String {
        self.ports
            .iter()
            .filter_map(|p| {
                p.public_port.map(|pub_port| {
                    format!("{}:{}", pub_port, p.private_port)
                })
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl DockerClient {
    pub async fn list_containers(&self, all: bool) -> Result<Vec<ContainerInfo>> {
        let docker = self.client()?;

        let options = ListContainersOptions::<String> {
            all,
            ..Default::default()
        };

        let containers = docker.list_containers(Some(options)).await?;

        let mut result = Vec::new();
        for container in containers {
            let id = container.id.unwrap_or_default();
            let names = container.names.unwrap_or_default();
            let name = names
                .first()
                .map(|n| n.trim_start_matches('/').to_string())
                .unwrap_or_else(|| id.clone());

            let ports = container
                .ports
                .unwrap_or_default()
                .into_iter()
                .map(|p| PortMapping {
                    private_port: p.private_port,
                    public_port: p.public_port,
                    protocol: p.typ.map(|t| t.to_string()).unwrap_or_else(|| "tcp".to_string()),
                    ip: p.ip,
                })
                .collect();

            let created = container
                .created
                .map(|ts| DateTime::from_timestamp(ts, 0))
                .flatten();

            result.push(ContainerInfo {
                id,
                name,
                image: container.image.unwrap_or_default(),
                image_id: container.image_id.unwrap_or_default(),
                state: ContainerState::from_str(
                    &container.state.unwrap_or_default(),
                ),
                status: container.status.unwrap_or_default(),
                created,
                ports,
                labels: container.labels.unwrap_or_default(),
                command: container.command,
                size_rw: container.size_rw,
                size_root_fs: container.size_root_fs,
            });
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }

    pub async fn start_container(&self, id: &str) -> Result<()> {
        let docker = self.client()?;
        docker
            .start_container(id, None::<StartContainerOptions<String>>)
            .await?;
        Ok(())
    }

    pub async fn stop_container(&self, id: &str) -> Result<()> {
        let docker = self.client()?;
        docker
            .stop_container(id, Some(StopContainerOptions { t: 10 }))
            .await?;
        Ok(())
    }

    pub async fn restart_container(&self, id: &str) -> Result<()> {
        let docker = self.client()?;
        docker
            .restart_container(id, Some(RestartContainerOptions { t: 10 }))
            .await?;
        Ok(())
    }

    pub async fn remove_container(&self, id: &str, force: bool) -> Result<()> {
        let docker = self.client()?;
        docker
            .remove_container(
                id,
                Some(RemoveContainerOptions {
                    force,
                    v: false,
                    link: false,
                }),
            )
            .await?;
        Ok(())
    }
}
