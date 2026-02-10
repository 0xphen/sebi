use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

use crate::report::model::{ArtifactHash, ArtifactInfo};

/// Raw artifact context used during analysis.
///
/// Holds the exact bytes analyzed and a cryptographic fingerprint
/// that uniquely identifies the artifact.
#[derive(Debug, Clone)]
pub struct ArtifactContext {
    /// Optional source path (informational only).
    pub path: Option<String>,

    /// Exact bytes read from disk.
    pub bytes: Vec<u8>,

    /// Size of the artifact in bytes.
    pub size_bytes: u64,

    /// Hash algorithm used for fingerprinting.
    pub hash_alg: String,

    /// Hex-encoded hash of the artifact bytes.
    pub hash_hex: String,
}

impl ArtifactContext {
    /// Convert into the public, report-facing artifact metadata.
    ///
    /// This intentionally drops raw bytes to prevent reuse after analysis.
    pub fn into_artifact(self) -> ArtifactInfo {
        ArtifactInfo {
            path: self.path,
            size_bytes: self.size_bytes,
            hash: ArtifactHash {
                algorithm: self.hash_alg,
                value: self.hash_hex,
            },
        }
    }
}

/// Read a WASM artifact and compute a stable cryptographic identity.
///
/// The identity depends **only** on the file bytes.
/// Filesystem metadata (timestamps, permissions, etc.) are ignored
/// to preserve deterministic analysis results.
pub fn read_artifact(path: &Path) -> Result<ArtifactContext> {
    let bytes =
        fs::read(path).with_context(|| format!("failed to read artifact: {}", path.display()))?;

    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let digest = hasher.finalize();

    Ok(ArtifactContext {
        path: Some(path.display().to_string()),
        size_bytes: bytes.len() as u64,
        bytes,
        hash_alg: "sha256".to_string(),
        hash_hex: hex::encode(digest),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn temp_artifact(data: &[u8]) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(data).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn reads_bytes_and_computes_stable_hash() {
        use std::io::Write;

        let data = b"sebi-test";

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(data).unwrap();
        file.flush().unwrap();

        let ctx = read_artifact(file.path()).expect("artifact read succeeds");

        assert_eq!(ctx.bytes, data);
        assert_eq!(ctx.size_bytes, data.len() as u64);
        assert_eq!(ctx.hash_alg, "sha256");

        // echo -n "sebi-test" | sha256sum
        assert_eq!(
            ctx.hash_hex,
            "2862ff95785ae5360e3308e9df61f0b4250a3137da4887f0c868279aa55432ba"
        );
    }

    #[test]
    fn different_inputs_produce_different_hashes() {
        let a = read_artifact(temp_artifact(b"data-a").path()).unwrap();
        let b = read_artifact(temp_artifact(b"data-b").path()).unwrap();

        assert_ne!(a.hash_hex, b.hash_hex);
    }

    #[test]
    fn missing_file_returns_error() {
        let result = read_artifact(Path::new("non_existent.wasm"));
        assert!(result.is_err());
    }

    #[test]
    fn converts_to_report_artifact() {
        let ctx = ArtifactContext {
            path: Some("test.wasm".into()),
            bytes: vec![0x00, 0x61, 0x73, 0x6d],
            size_bytes: 4,
            hash_alg: "sha256".into(),
            hash_hex: "abcd".into(),
        };

        let artifact = ctx.into_artifact();
        assert_eq!(artifact.path, Some("test.wasm".into()));
        assert_eq!(artifact.hash.value, "abcd");
    }
}
