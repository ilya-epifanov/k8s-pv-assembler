use std::{sync::Arc, time::Duration};

use futures::StreamExt;
use kube::Resource;
use kube::ResourceExt;
use kube::{
    api::ListParams,
    runtime::{controller::Action, Controller},
    Api, Client,
};

mod crd;
pub mod db;
mod finalizer;
mod pvc_part;

use crd::PvPart;
use sled::transaction::{ConflictableTransactionError, TransactionError};
use tracing::{info, warn};

use crate::opts::ConfigV1;

use self::db::Db;

pub async fn controller(client: Client, config: &ConfigV1, db: Db) -> Result<(), anyhow::Error> {
    let crd_api: Api<PvPart> = Api::all(client.clone());
    let context: Arc<ContextData> = Arc::new(ContextData::new(client.clone(), config.clone(), db));

    Controller::new(crd_api.clone(), ListParams::default())
        .run(reconcile, on_error, context)
        .for_each(|reconciliation_result| async move {
            match reconciliation_result {
                Ok(echo_resource) => {
                    info!("reconciliation successful. Resource: {:?}", echo_resource);
                }
                Err(reconciliation_err) => {
                    warn!("reconciliation error: {:?}", reconciliation_err)
                }
            }
        })
        .await;

    Ok(())
}

struct ContextData {
    client: Client,
    config: ConfigV1,
    db: Db,
}

impl ContextData {
    pub fn new(client: Client, config: ConfigV1, db: Db) -> Self {
        ContextData { client, config, db }
    }
}

enum PvPartAction {
    Update,
    Delete,
    NoOp,
}

async fn reconcile(pv_part: Arc<PvPart>, context: Arc<ContextData>) -> Result<Action, Error> {
    let client: Client = context.client.clone();

    let namespace: String = pv_part.namespace().ok_or(Error::ResourceIsNotNamespaced)?;
    let name = pv_part.name_any();

    return match determine_action(&pv_part) {
        PvPartAction::Update => {
            finalizer::add(client.clone(), &name, &namespace).await?;
            pvc_part::deploy(
                &pv_part.spec,
                &context.config,
                &namespace,
                &name,
                &context.db,
            )
            .await?;
            Ok(Action::requeue(Duration::from_secs(10)))
        }
        PvPartAction::Delete => {
            pvc_part::delete(
                &pv_part.spec,
                &context.config,
                &namespace,
                &name,
                &context.db,
            )
            .await?;
            finalizer::delete(client, &name, &namespace).await?;
            Ok(Action::await_change())
        }
        PvPartAction::NoOp => Ok(Action::requeue(Duration::from_secs(10))),
    };
}

fn determine_action(pv_part: &PvPart) -> PvPartAction {
    if pv_part.meta().deletion_timestamp.is_some() {
        PvPartAction::Delete
    } else {
        PvPartAction::Update
        // } else {
        //     PvPartAction::NoOp
    }
}

fn on_error(pv_part: Arc<PvPart>, error: &Error, _context: Arc<ContextData>) -> Action {
    warn!("reconciliation error:\n{:?}.\n{:?}", error, pv_part);
    Action::requeue(Duration::from_secs(5))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("kubernetes reported error: {0}")]
    KubeError(#[from] kube::Error),
    #[error("volume {0} is not configured")]
    UnknownVolume(String),
    #[error("volume {0} is not mounted")]
    VolumeNotMounted(String),
    #[error("pv_part in namespace {0} isn't allowed to be merged into volume {1}")]
    Forbidden(String, String),
    #[error("pv_part is not namespaced")]
    ResourceIsNotNamespaced,
    #[error("malformed filename")]
    FileName,
    #[error("I/O error during synchronization")]
    Io(#[from] std::io::Error),
    #[error("file that {0} wanted to create is already owned by {1}")]
    Conflict(String, String),
    #[error("database is corrupted")]
    DbDataError,
    #[error("can't keep a record of reconciliation: {0}")]
    TxError(#[from] sled::Error),
}

impl From<TransactionError<Error>> for Error {
    fn from(e: TransactionError<Error>) -> Self {
        match e {
            TransactionError::Abort(e) => e,
            TransactionError::Storage(e) => Error::TxError(e),
        }
    }
}

impl Into<ConflictableTransactionError<Error>> for Error {
    fn into(self) -> ConflictableTransactionError<Error> {
        ConflictableTransactionError::Abort(self)
    }
}
