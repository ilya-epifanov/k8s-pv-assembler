use std::collections::HashMap;

use kube::CustomResource;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(CustomResource, Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
#[kube(
    group = "ilya-epifanov.github.com",
    version = "v1",
    kind = "PvPart",
    singular = "pv-part",
    plural = "pv-parts",
    derive = "PartialEq",
    status = "PvPartStatus",
    namespaced
)]
pub struct PvPartSpec {
    pub target_volume: String,
    pub files: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, JsonSchema)]
pub enum PvPartStatus {
    Absent,
    Present,
}

#[cfg(test)]
mod tests {
    use std::{fs::File, path::Path};

    use kube::CustomResourceExt;

    use super::*;

    #[test]
    fn generate_crd() -> Result<(), anyhow::Error> {
        let crd_yaml_file = File::create(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("charts/pv-assembler/crds")
                .join("crd.yaml"),
        )?;
        serde_yaml::to_writer(crd_yaml_file, &PvPart::crd())?;
        Ok(())
    }
}
