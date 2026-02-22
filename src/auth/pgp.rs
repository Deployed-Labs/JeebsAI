use openpgp::cert::Cert;
use openpgp::parse::Parse;
use openpgp::policy::StandardPolicy;
use sequoia_openpgp as openpgp;
use std::io::Cursor;

/// Hardcoded PGP public key for the 1090mb super-admin account.
/// This key is the ultimate authority — it cannot be overridden by any other admin.
pub const ROOT_ADMIN_PGP_KEY: &str = r#"-----BEGIN PGP PUBLIC KEY BLOCK-----

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
=Wh64
-----END PGP PUBLIC KEY BLOCK-----"#;

pub fn validate_public_key(public_key: &str) -> Result<(), String> {
    Cert::from_bytes(public_key.as_bytes())
        .map(|_| ())
        .map_err(|e| format!("Invalid PGP public key: {e}"))
}

pub fn verify_signature_with_public_key(
    signed_message: &str,
    public_key: &str,
) -> Result<String, String> {
    let cert = Cert::from_bytes(public_key.as_bytes())
        .map_err(|e| format!("Invalid PGP public key: {e}"))?;
    let policy = StandardPolicy::new();

    let mut message_reader = Cursor::new(signed_message.as_bytes());
    let helper = Helper { cert };
    let mut verifier = openpgp::parse::stream::VerifierBuilder::from_reader(&mut message_reader)
        .map_err(|e| format!("Failed to create verifier: {e}"))?
        .with_policy(&policy, None, helper)
        .map_err(|e| format!("Failed to verify: {e}"))?;

    let mut verified_data = Vec::new();
    std::io::copy(&mut verifier, &mut verified_data)
        .map_err(|e| format!("Failed to read verified data: {e}"))?;

    String::from_utf8(verified_data)
        .map_err(|e| format!("Failed to convert verified data to UTF-8: {e}"))
}

struct Helper {
    cert: Cert,
}

impl openpgp::parse::stream::VerificationHelper for Helper {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> openpgp::Result<Vec<Cert>> {
        Ok(vec![self.cert.clone()])
    }

    fn check(
        &mut self,
        structure: openpgp::parse::stream::MessageStructure,
    ) -> openpgp::Result<()> {
        use std::io::{Error, ErrorKind};

        for layer in structure.iter() {
            if let openpgp::parse::stream::MessageLayer::SignatureGroup { results } = layer {
                for result in results {
                    if result.is_ok() {
                        return Ok(());
                    }
                }
                return Err(Error::new(ErrorKind::InvalidData, "Bad signature").into());
            }
        }

        Err(Error::new(ErrorKind::InvalidData, "No valid signature found").into())
    }
}

impl openpgp::parse::stream::DecryptionHelper for Helper {
    fn decrypt<D>(
        &mut self,
        _: &[openpgp::packet::PKESK],
        _: &[openpgp::packet::SKESK],
        _: Option<openpgp::types::SymmetricAlgorithm>,
        _: D,
    ) -> openpgp::Result<Option<openpgp::Fingerprint>>
    where
        D: FnMut(openpgp::types::SymmetricAlgorithm, &openpgp::crypto::SessionKey) -> bool,
    {
        Ok(None)
    }
}
