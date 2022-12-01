use std::collections::BTreeSet;

use itertools::Itertools;
use sled::transaction::ConflictableTransactionError;
use tracing::{debug, info};

use super::Error;

pub struct Db {
    db: sled::Db,
}

impl Db {
    pub fn new(db: sled::Db) -> Self {
        Self { db }
    }

    fn owner_key(namespace: &str, name: &str) -> String {
        format!("owner:{namespace}/{name}")
    }

    fn file_key(path: &str) -> String {
        format!("path:{path}")
    }

    pub fn apply_changes(
        &self,
        owner_namespace: &str,
        owner_name: &str,
        target_paths: &[&str],
        f: impl Fn(&[&str]) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let owner = Self::owner_key(owner_namespace, owner_name);

        self.db.transaction(|db| {
            info!("checking file ownership");
            for path in target_paths {
                if let Some(file_data) = db.get(Self::file_key(path))? {
                    if file_data != &owner {
                        return Err(Error::Conflict(
                            String::from_utf8_lossy(&file_data).into_owned(),
                            owner.to_owned(),
                        )
                        .into());
                    }
                }
            }

            let old_ownesrship_list = db.get(&owner)?;

            let mut old_files = BTreeSet::new();

            if let Some(old_ownesrship_list) = old_ownesrship_list.as_ref() {
                for owned_file in old_ownesrship_list.split(|&b| b == 0) {
                    let owned_file = std::str::from_utf8(owned_file)
                        .map_err(|_| ConflictableTransactionError::Abort(Error::DbDataError))?;
                    old_files.insert(owned_file);
                }
            }
            let new_files: BTreeSet<&str> = target_paths.iter().cloned().collect();

            debug!("old file list: [{:?}]", &old_files);
            debug!("new file list: [{:?}]", &new_files);

            let to_delete = old_files.difference(&new_files);
            for deleted_path in to_delete.clone() {
                db.remove(Self::file_key(deleted_path).as_bytes())?;
            }
            for added_path in new_files.difference(&old_files) {
                db.insert(Self::file_key(added_path).as_bytes(), owner.as_bytes())?;
            }
            db.insert(owner.as_bytes(), target_paths.join("\0").as_bytes())?;

            f(&to_delete.into_iter().cloned().collect_vec())
                .map_err(Into::<ConflictableTransactionError<_>>::into)?;

            Ok(())
        })?;
        Ok(())
    }

    pub fn remove(
        &self,
        owner_namespace: &str,
        owner_name: &str,
        f: impl Fn(&[&str]) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let owner = Self::owner_key(owner_namespace, owner_name);

        self.db.transaction(|db| {
            let mut to_delete = vec![];
            let old_ownership_list = db.get(&owner)?;
            if let Some(old_ownership_list) = old_ownership_list.as_ref() {
                for owned_file in old_ownership_list.split(|&b| b == 0) {
                    let owned_file = std::str::from_utf8(owned_file)
                        .map_err(|_| ConflictableTransactionError::Abort(Error::DbDataError))?;
                    to_delete.push(owned_file);
                }
            }

            f(&to_delete).map_err(Into::<ConflictableTransactionError<_>>::into)?;

            Ok(())
        })?;

        Ok(())
    }
}
