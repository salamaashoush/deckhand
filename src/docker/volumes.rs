use anyhow::Result;
use bollard::volume::{ListVolumesOptions, RemoveVolumeOptions};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::DockerClient;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
    pub created: Option<DateTime<Utc>>,
    pub labels: HashMap<String, String>,
    pub scope: String,
    pub status: Option<HashMap<String, String>>,
    pub usage_data: Option<VolumeUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeUsage {
    pub size: i64,
    pub ref_count: i64,
}

impl VolumeInfo {
    pub fn display_size(&self) -> String {
        self.usage_data
            .as_ref()
            .map(|u| bytesize::ByteSize(u.size as u64).to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    pub fn is_in_use(&self) -> bool {
        self.usage_data
            .as_ref()
            .map(|u| u.ref_count > 0)
            .unwrap_or(false)
    }
}

impl DockerClient {
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>> {
        let docker = self.client()?;

        let options = ListVolumesOptions::<String> {
            ..Default::default()
        };

        let response = docker.list_volumes(Some(options)).await?;

        let volumes = response.volumes.unwrap_or_default();
        let mut result = Vec::new();

        for volume in volumes {
            let created = volume
                .created_at
                .as_ref()
                .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            let usage_data = volume.usage_data.map(|u| VolumeUsage {
                size: u.size,
                ref_count: u.ref_count,
            });

            result.push(VolumeInfo {
                name: volume.name,
                driver: volume.driver,
                mountpoint: volume.mountpoint,
                created,
                labels: volume.labels,
                scope: volume.scope.map(|s| format!("{:?}", s)).unwrap_or_default(),
                status: None,
                usage_data,
            });
        }

        result.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(result)
    }

    pub async fn remove_volume(&self, name: &str, force: bool) -> Result<()> {
        let docker = self.client()?;
        docker
            .remove_volume(name, Some(RemoveVolumeOptions { force }))
            .await?;
        Ok(())
    }

    pub async fn create_volume(&self, name: &str) -> Result<VolumeInfo> {
        let docker = self.client()?;

        let config = bollard::volume::CreateVolumeOptions {
            name: name.to_string(),
            ..Default::default()
        };

        let volume = docker.create_volume(config).await?;

        let created = volume
            .created_at
            .as_ref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(VolumeInfo {
            name: volume.name,
            driver: volume.driver,
            mountpoint: volume.mountpoint,
            created,
            labels: volume.labels,
            scope: volume.scope.map(|s| format!("{:?}", s)).unwrap_or_default(),
            status: None,
            usage_data: None,
        })
    }
}
