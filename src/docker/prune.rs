use anyhow::Result;
use bollard::container::PruneContainersOptions;
use bollard::image::PruneImagesOptions;
use bollard::network::PruneNetworksOptions;
use bollard::volume::PruneVolumesOptions;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::DockerClient;

/// Result of a prune operation
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PruneResult {
  pub containers_deleted: Vec<String>,
  pub images_deleted: Vec<String>,
  pub volumes_deleted: Vec<String>,
  pub networks_deleted: Vec<String>,
  pub space_reclaimed: u64,
}

impl PruneResult {
  pub fn display_space_reclaimed(&self) -> String {
    bytesize::ByteSize(self.space_reclaimed).to_string()
  }

  pub fn total_items_deleted(&self) -> usize {
    self.containers_deleted.len() + self.images_deleted.len() + self.volumes_deleted.len() + self.networks_deleted.len()
  }

  pub fn is_empty(&self) -> bool {
    self.total_items_deleted() == 0
  }
}

impl DockerClient {
  /// Prune stopped containers
  pub async fn prune_containers(&self) -> Result<PruneResult> {
    let docker = self.client()?;

    let options = PruneContainersOptions::<String> {
      filters: HashMap::new(),
    };

    let response = docker.prune_containers(Some(options)).await?;

    Ok(PruneResult {
      containers_deleted: response.containers_deleted.unwrap_or_default(),
      space_reclaimed: response.space_reclaimed.unwrap_or(0) as u64,
      ..Default::default()
    })
  }

  /// Prune unused images
  pub async fn prune_images(&self, dangling_only: bool) -> Result<PruneResult> {
    let docker = self.client()?;

    let mut filters = HashMap::new();
    if dangling_only {
      filters.insert("dangling", vec!["true"]);
    }

    let options = PruneImagesOptions { filters };

    let response = docker.prune_images(Some(options)).await?;

    let images_deleted = response
      .images_deleted
      .unwrap_or_default()
      .into_iter()
      .filter_map(|item| item.untagged.or(item.deleted))
      .collect();

    Ok(PruneResult {
      images_deleted,
      space_reclaimed: response.space_reclaimed.unwrap_or(0) as u64,
      ..Default::default()
    })
  }

  /// Prune unused volumes
  pub async fn prune_volumes(&self) -> Result<PruneResult> {
    let docker = self.client()?;

    let options = PruneVolumesOptions::<String> {
      filters: HashMap::new(),
    };

    let response = docker.prune_volumes(Some(options)).await?;

    Ok(PruneResult {
      volumes_deleted: response.volumes_deleted.unwrap_or_default(),
      space_reclaimed: response.space_reclaimed.unwrap_or(0) as u64,
      ..Default::default()
    })
  }

  /// Prune unused networks
  pub async fn prune_networks(&self) -> Result<PruneResult> {
    let docker = self.client()?;

    let options = PruneNetworksOptions::<String> {
      filters: HashMap::new(),
    };

    let response = docker.prune_networks(Some(options)).await?;

    Ok(PruneResult {
      networks_deleted: response.networks_deleted.unwrap_or_default(),
      ..Default::default()
    })
  }
}
