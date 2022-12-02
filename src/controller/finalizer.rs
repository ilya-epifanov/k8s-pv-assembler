use super::crd::PvPart;
use kube::api::{Patch, PatchParams};
use kube::{Api, Client, Error};
use serde_json::{json, Value};
use tracing::debug;

pub async fn add(client: Client, name: &str, namespace: &str) -> Result<PvPart, Error> {
    debug!("adding finalizer");
    let api: Api<PvPart> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": ["pv-parts.ilya-epifanov.github.com/finalizer"]
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    Ok(api.patch(name, &PatchParams::default(), &patch).await?)
}

pub async fn delete(client: Client, name: &str, namespace: &str) -> Result<PvPart, Error> {
    debug!("removing finalizer");
    let api: Api<PvPart> = Api::namespaced(client, namespace);
    let finalizer: Value = json!({
        "metadata": {
            "finalizers": null
        }
    });

    let patch: Patch<&Value> = Patch::Merge(&finalizer);
    Ok(api.patch(name, &PatchParams::default(), &patch).await?)
}
