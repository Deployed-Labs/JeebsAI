use sequoia_openpgp as openpgp;
use openpgp::parse::Parse;
use openpgp::policy::StandardPolicy;
use openpgp::cert::Cert;
use std::io::Cursor;
use once_cell::sync::Lazy;

pub const ADMIN_PGP_PUBLIC_KEY: &str = r#"-----BEGIN PGP PUBLIC KEY BLOCK-----
Version: openpgp-mobile

xsBNBGmVKw8BCAC8ooVLnnJa0CZ1QhRWK/XhycvjhNKbV9jWUMY9n6Vh06B0iWGo
wqQy7H9LjP3IMUaIcjQRJlAQJFNZxcO0d2FjTjHEeHDMGNALAhjtFwfL7r1TXTMG
BE6vSVJlrnfCnCMEyIbhT6ntCrkozKEUinz36sKl9wLoG+F7Otlo5Eg5uDttme+5
NLmX7yCFgyiv18sUCbIXVxGVOBqh4XStTtbk9bOTkd8Pe/MtqTU/pJABbihu8YxO
+LFPj7C20UcjChtOYVKan3ym1vzA/JAizh/aF/GEy295FGwdLSh4jUoHQJXKrK0+
BHLxQ1xzmCoPldpRZUuLgTV8Y3YkxjTF4XQPABEBAAHNLDEwOTBtYl9KZWVic0FJ
X2FkbWluIDwxMDkwbWJAcHJvdG9ubWFpbC5jb20+wsCKBBMBCAA+BQJplSsPCZBZ
0nyPo+Q65xYhBMiHO0V98s0hvxjeqFnSfI+j5DrnAhsDAh4BAhkBAwsJBwIVCAMW
AAICIgEAACeHCACMLzj9+XeBPvUoISnYUIIj9AxY5jJs2E86fI8mcKx3elT51qx0
UpKa8TJ9VZnFEBGSspwJ+5bFz0fwgivBZr2cKmikUjoKrJVzIuBMfYts7bU9WvXV
lMWS/jIqM4MLwqWwvYFiwSQGftMDjDUqPpg+Jakkug5mwHqbbtjtvaHA47/d3GOU
dwc9B2l1+I3cj4EkkY+SphIHCKI74jltEjvNXY8TwH+ZUssOw706i9ncCkAwdyp3
7qKxxnrFznEGpQwqXYs3bO2YhW5PlwgyNlKOX6mxQb/EocznpHJGFr0Bg2rQuRoo
LR5bpJsRQFLng3nahtZK5rCWBsNayOhtbi2OzsBNBGmVKw8BCADvcAQbOcjp9Yvr
dnJRfaTb0t4FDjPg52ueeAc/Xbqd34wYfBIqKDtkOjlGlIJaSZt8z0kCTPaHSzOZ
DorF31qPxiUlXmZUgTwb6HoTqMm9n8NobEclgpSg0BlMvvqNxYP5FyLEvyGKfW4J
jotYoecV5PsLkZThMGZunFNav6e0TiDNOWFFzwP+p8NucJqsk/yCW7MQAvacHP5A
Lhc0flZJV+La14ltgHebZ7AI2b8iOBZXtP/0mpTwdWsPOmyhUexVB+KMarvgvXBY
WN87U+62f8zurPGkQxna/Xr0118lKumj1WbClvNd5JFlSnN4SgPv2SOgms5ntNN6
J/N0umfVABEBAAHCwHYEGAEIACoFAmmVKw8JkFnSfI+j5DrnFiEEyIc7RX3yzSG/
GN6oWdJ8j6PkOucCGwwAABncB/9YzsoJ1+rmlCTh3xYkpZepbcnS2V7k1EecgNe4
7/reTWf+8XT9pkwYjbAzoZpx8uXzX6uwkykPxnhziIRu2LkbjsmnuJoIwMyXYOfm
dxLlu/2YVJDZ3yUbJzwUXDhAh1X5hQW5BfCv0AGHDWGWriWk5Fi9WYkrhomBL1tY
GxkgTnvZyTj7/QoTeqK2ko+Ww5T6wfYYKtQu4Mpm7QZCokEZR8DNYAyNJ6TIMVzL
Tadii9qkpPDIcxSITjmhbzLQbcshC2rxxo2nGD4KOuEzys7hqlU+0Tx97gonl8bx
1MB/gMBp9i1q1huZeXYczhJV/5t6KCN1WruqatUqBbcn5T2J
=xnit
-----END PGP PUBLIC KEY BLOCK-----"#;

// Parse and cache the certificate once
static ADMIN_CERT: Lazy<Cert> = Lazy::new(|| {
    Cert::from_bytes(ADMIN_PGP_PUBLIC_KEY.as_bytes())
        .expect("Failed to parse admin PGP public key")
});

/// Verify a signed message using the admin's PGP public key
pub fn verify_signature(signed_message: &str) -> Result<String, String> {
    let policy = StandardPolicy::new();
    
    // Parse the signed message
    let message_bytes = signed_message.as_bytes();
    let mut message_reader = Cursor::new(message_bytes);
    
    // Verify the signature using cached certificate
    let helper = Helper::new(&ADMIN_CERT);
    let mut verifier = openpgp::parse::stream::VerifierBuilder::from_reader(&mut message_reader)
        .map_err(|e| format!("Failed to create verifier: {}", e))?
        .with_policy(&policy, None, helper)
        .map_err(|e| format!("Failed to verify: {}", e))?;
    
    // Read the verified message
    let mut verified_data = Vec::new();
    std::io::copy(&mut verifier, &mut verified_data)
        .map_err(|e| format!("Failed to read verified data: {}", e))?;
    
    String::from_utf8(verified_data)
        .map_err(|e| format!("Failed to convert verified data to string: {}", e))
}

struct Helper<'a> {
    cert: &'a Cert,
}

impl<'a> Helper<'a> {
    fn new(cert: &'a Cert) -> Self {
        Helper { cert }
    }
}

impl<'a> openpgp::parse::stream::VerificationHelper for Helper<'a> {
    fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> openpgp::Result<Vec<Cert>> {
        Ok(vec![self.cert.clone()])
    }

    fn check(&mut self, structure: openpgp::parse::stream::MessageStructure) -> openpgp::Result<()> {
        use std::io::{Error, ErrorKind};
        for layer in structure.iter() {
            match layer {
                openpgp::parse::stream::MessageLayer::SignatureGroup { results } => {
                    for result in results {
                        if result.is_ok() {
                            return Ok(());
                        }
                    }
                    return Err(Error::new(ErrorKind::InvalidData, "Bad signature").into());
                }
                _ => {}
            }
        }
        Err(Error::new(ErrorKind::InvalidData, "No valid signature found").into())
    }
}

impl<'a> openpgp::parse::stream::DecryptionHelper for Helper<'a> {
    fn decrypt<D>(&mut self, _: &[openpgp::packet::PKESK], _: &[openpgp::packet::SKESK], _: Option<openpgp::types::SymmetricAlgorithm>, _: D) -> openpgp::Result<Option<openpgp::Fingerprint>>
    where
        D: FnMut(openpgp::types::SymmetricAlgorithm, &openpgp::crypto::SessionKey) -> bool,
    {
        Ok(None)
    }
}
