use itertools::Itertools;
use std::collections::BTreeMap;
use std::path::{Component, Path};
use tracing::{debug, warn};

use crate::opts::Opts;

use super::crd::PvPartSpec;
use super::db::Db;
use super::Error;

pub async fn deploy(
    spec: &PvPartSpec,
    config: &Opts,
    namespace: &str,
    name: &str,
    db: &Db,
) -> Result<(), Error> {
    debug!("deploying pv-part");
    if !config.volumes.contains(&spec.target_volume) {
        return Err(Error::VolumeNotMounted(spec.target_volume.to_owned()));
    }

    if config.namespace != namespace {
        return Err(Error::Forbidden(
            namespace.to_owned(),
            spec.target_volume.to_owned(),
        ));
    }

    let volume_path = Path::new("/volumes").join(&spec.target_volume);
    if !volume_path.is_dir() {
        return Err(Error::VolumeNotMounted(spec.target_volume.to_owned()));
    }

    let mut files = BTreeMap::new();
    for (k, v) in &spec.files {
        let k = Path::new(k);
        for component in k.components() {
            if !matches!(component, Component::Normal(_)) {
                warn!("file path isn't a simple relative path");
                return Err(Error::FileName);
            }
        }

        let k = volume_path
            .join(k)
            .into_os_string()
            .into_string()
            .map_err(|_| Error::FileName)?;
        files.insert(k, v);
    }

    for file_path in files.keys() {
        if !Path::new(file_path).starts_with(&volume_path) {
            warn!("path traversal attempt detected");
            return Err(Error::FileName);
        }
    }

    db.apply_changes(
        namespace,
        name,
        &files.keys().map(|k| k.as_str()).collect_vec(),
        |to_delete| {
            for (file, contents) in files.iter() {
                let file_path = volume_path.join(file);

                if let Some(parent) = file_path.parent() {
                    debug!("creating direcroties for {file_path:?}: {parent:?}");
                    std::fs::create_dir_all(parent)?;
                }

                debug!("writing file {file_path:?}");
                std::fs::write(&file_path, contents)?;

                for file_to_delete in to_delete {
                    let path = Path::new(file_to_delete);
                    assert!(path.starts_with(&volume_path));

                    debug!("removing file {path:?}");
                    if path.is_file() {
                        std::fs::remove_file(path)?;
                    }
                    let parent_dir = path.parent();
                    if let Some(parent_dir) = parent_dir {
                        let parent_dir = parent_dir.canonicalize()?;
                        if parent_dir.starts_with(&volume_path) && parent_dir != volume_path {
                            debug!("removing potentially empty directory {parent_dir:?}");
                            let _ = std::fs::remove_dir(parent_dir);
                        }
                    }
                }
            }
            Ok(())
        },
    )?;

    Ok(())
}

pub async fn delete(
    spec: &PvPartSpec,
    config: &Opts,
    namespace: &str,
    name: &str,
    db: &Db,
) -> Result<(), Error> {
    debug!("deleting pv-part");
    if !config.volumes.contains(&spec.target_volume) {
        return Err(Error::VolumeNotMounted(spec.target_volume.to_owned()));
    }

    if config.namespace != namespace {
        return Err(Error::Forbidden(
            namespace.to_owned(),
            spec.target_volume.to_owned(),
        ));
    }

    let volume_path = Path::new("/volumes").join(&spec.target_volume);
    if !volume_path.is_dir() {
        return Err(Error::VolumeNotMounted(spec.target_volume.to_owned()));
    }

    db.remove(namespace, name, |to_delete| {
        for file_to_delete in to_delete {
            let path = Path::new(file_to_delete);
            assert!(path.starts_with(&volume_path));

            debug!("removing file {path:?}");
            if path.is_file() {
                std::fs::remove_file(path)?;
            }
            let parent_dir = path.parent();
            if let Some(parent_dir) = parent_dir {
                let parent_dir = parent_dir.canonicalize()?;
                if parent_dir.starts_with(&volume_path) && parent_dir != volume_path {
                    debug!("removing potentially empty directory {parent_dir:?}");
                    let _ = std::fs::remove_dir(parent_dir);
                }
            }
        }

        Ok(())
    })?;

    Ok(())
}
