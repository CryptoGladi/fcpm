use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct RepositoryManifest {
    name: String,
    packages: Vec<String>,
}
