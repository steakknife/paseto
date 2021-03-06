use errors::*;
#[cfg(feature = "v1")]
use v1::public_paseto as V1Public;
#[cfg(all(not(feature = "v2"), feature = "v1"))]
use v1::local_paseto as V1Local;
#[cfg(feature = "v2")]
use v2::{local_paseto as V2Local, public_paseto as V2Public};

use chrono::prelude::*;
#[cfg(feature = "v1")]
use ring::signature::{RSAKeyPair, RSASigningState};
#[cfg(feature = "v2")]
use ring::signature::Ed25519KeyPair;
use serde_json::{Value as JsonValue, to_string as JsonToString};
#[cfg(feature = "v1")]
use untrusted::Input as UntrustedInput;

use std::collections::HashMap;
#[cfg(feature = "v1")]
use std::sync::Arc;

/// A paseto builder.
pub struct PasetoBuilder {
  /// Set the footer to use for this token.
  footer: Option<String>,
  /// The encryption key to use. If present WILL use LOCAL tokens (or shared key encryption).
  encryption_key: Option<Vec<u8>>,
  /// The RSA Key pairs in DER format, for V1 Public Tokens.
  #[cfg(feature = "v1")]
  rsa_key: Option<(Vec<u8>)>,
  /// The ED25519 Key Pair, for V2 Public Tokens.
  #[cfg(feature = "v2")]
  ed_key: Option<Ed25519KeyPair>,
  /// Any extra claims you want to store in your json.
  extra_claims: HashMap<String, JsonValue>,
}

#[cfg(all(feature = "v1", feature = "v2"))]
impl PasetoBuilder {
  /// Creates a new Paseto builder.
  pub fn new() -> PasetoBuilder {
    PasetoBuilder {
      footer: None,
      encryption_key: None,
      rsa_key: None,
      ed_key: None,
      extra_claims: HashMap::new(),
    }
  }

  /// Builds a token.
  pub fn build(self) -> Result<String> {
    let json_as_str = JsonToString(&self.extra_claims);
    if json_as_str.is_err() {
      return Err(ErrorKind::JsonError.into());
    }
    let strd_msg = json_as_str.unwrap();

    if self.encryption_key.is_some() {
      let mut enc_key = self.encryption_key.unwrap();
      return V2Local(strd_msg, self.footer, &mut enc_key);
    } else if self.ed_key.is_some() {
      let ed_key_pair = self.ed_key.unwrap();
      return V2Public(strd_msg, self.footer, &ed_key_pair);
    } else if self.rsa_key.is_some() {
      let the_rsa_key = self.rsa_key.unwrap();
      let private_key_der = UntrustedInput::from(&the_rsa_key);
      let key_pair = RSAKeyPair::from_der(private_key_der);
      if key_pair.is_err() {
        return Err(ErrorKind::InvalidRsaKey.into());
      }
      let key_pair = Arc::new(key_pair.unwrap());
      let signing_state = RSASigningState::new(key_pair);
      if signing_state.is_err() {
        return Err(ErrorKind::InvalidRsaKey.into());
      }
      let mut signing_state = signing_state.unwrap();
      return V1Public(strd_msg, self.footer, &mut signing_state);
    } else {
      return Err(ErrorKind::NoKeysProvided.into());
    }
  }
}

#[cfg(all(not(feature = "v2"), feature = "v1"))]
impl PasetoBuilder {
  /// Creates a new Paseto builder.
  pub fn new() -> PasetoBuilder {
    PasetoBuilder {
      footer: None,
      encryption_key: None,
      rsa_key: None,
      extra_claims: HashMap::new(),
    }
  }

  /// Builds a token.
  pub fn build(self) -> Result<String> {
    let json_as_str = JsonToString(&self.extra_claims);
    if json_as_str.is_err() {
      return Err(ErrorKind::JsonError.into());
    }
    let strd_msg = json_as_str.unwrap();

    if self.encryption_key.is_some() {
      let mut enc_key = self.encryption_key.unwrap();
      return V1Local(strd_msg, self.footer, &mut self.enc_key);
    } else if self.rsa_key.is_some() {
      let the_rsa_key = self.rsa_key.unwrap();
      let private_key_der = UntrustedInput::from(&the_rsa_key);
      let key_pair = RSAKeyPair::from_der(private_key_der);
      if key_pair.is_err() {
        return Err(ErrorKind::InvalidRsaKey.into());
      }
      let key_pair = Arc::new(key_pair.unwrap());
      let signing_state = RSASigningState::new(key_pair);
      if signing_state.is_err() {
        return Err(ErrorKind::InvalidRsaKey.into());
      }
      let mut signing_state = signing_state.unwrap();
      return V1Public(strd_msg, self.footer, &mut signing_state);
    } else {
      return Err(ErrorKind::NoKeysProvided.into());
    }
  }
}

#[cfg(all(not(feature = "v1"), feature = "v2"))]
impl PasetoBuilder {
  /// Creates a new Paseto builder.
  pub fn new() -> PasetoBuilder {
    PasetoBuilder {
      footer: None,
      encryption_key: None,
      ed_key: None,
      extra_claims: HashMap::new(),
    }
  }

  /// Builds a token.
  pub fn build(self) -> Result<String> {
    let json_as_str = JsonToString(&self.extra_claims);
    if json_as_str.is_err() {
      return Err(ErrorKind::JsonError.into());
    }
    let strd_msg = json_as_str.unwrap();

    if self.encryption_key.is_some() {
      let mut enc_key = self.encryption_key.unwrap();
      return V2Local(strd_msg, self.footer, &mut enc_key);
    } else if self.ed_key.is_some() {
      let ed_key_pair = self.ed_key.unwrap();
      return V2Public(strd_msg, self.footer, &ed_key_pair);
    } else {
      return Err(ErrorKind::NoKeysProvided.into());
    }
  }
}

#[cfg(feature = "v1")]
impl PasetoBuilder {
  /// Sets the RSA Key on a Paseto builder.
  ///
  /// NOTE: This will not be used if you set a symmetric encryption key, or if you specify an Ed25519 key pair.
  pub fn set_rsa_key(mut self, private_key_der: Vec<u8>) -> Self {
    self.rsa_key = Some((private_key_der));
    self
  }
}

#[cfg(feature = "v2")]
impl PasetoBuilder {
  /// Sets the ED25519 Key pair.
  ///
  /// NOTE: This will not be used if you set a symmetric encryption key.
  pub fn set_ed25519_key(mut self, key_pair: Ed25519KeyPair) -> Self {
    self.ed_key = Some(key_pair);
    self
  }
}

impl PasetoBuilder {
  /// Sets the encryption key to use for the paseto token.
  ///
  /// NOTE: If you set this we _*will*_ use a local token.
  pub fn set_encryption_key(mut self, encryption_key: Vec<u8>) -> Self {
    self.encryption_key = Some(encryption_key);
    self
  }

  //// Sets the footer to use for this token.
  pub fn set_footer(mut self, footer: String) -> Self {
    self.footer = Some(footer);
    self
  }

  /// Sets an arbitrary claim (a key inside the json token).
  pub fn set_claim(mut self, key: String, value: JsonValue) -> Self {
    self.extra_claims.insert(key, value);
    self
  }

  /// Sets the audience for this token.
  pub fn set_audience(self, audience: String) -> Self {
    self.set_claim(String::from("aud"), json!(audience))
  }

  /// Sets the expiration date for this token.
  pub fn set_expiration(self, expiration: DateTime<Utc>) -> Self {
    self.set_claim(String::from("exp"), json!(expiration))
  }

  /// Sets the time this token was issued at.
  ///
  /// issued_at defaults to: Utc::now();
  pub fn set_issued_at(self, issued_at: Option<DateTime<Utc>>) -> Self {
    self.set_claim(String::from("iat"), json!(issued_at.unwrap_or(Utc::now())))
  }

  /// Sets the issuer for this token.
  pub fn set_issuer(self, issuer: String) -> Self {
    self.set_claim(String::from("iss"), json!(issuer))
  }

  /// Sets the JTI ID for this token.
  pub fn set_jti(self, id: String) -> Self {
    self.set_claim(String::from("jti"), json!(id))
  }

  /// Sets the not before time.
  pub fn set_not_before(self, not_before: DateTime<Utc>) -> Self {
    self.set_claim(String::from("nbf"), json!(not_before))
  }

  /// Sets the subject for this token.
  pub fn set_subject(self, subject: String) -> Self {
    self.set_claim(String::from("sub"), json!(subject))
  }
}

#[cfg(test)]
mod unit_test {
  use super::*;
  use v2::local::decrypt_paseto as V2Decrypt;

  use serde_json::from_str as ParseJson;

  #[test]
  fn can_construct_a_token() {
    let token = PasetoBuilder::new()
      .set_encryption_key(Vec::from("YELLOW SUBMARINE, BLACK WIZARDRY".as_bytes()))
      .set_issued_at(None)
      .set_expiration(Utc::now())
      .set_issuer(String::from("issuer"))
      .set_audience(String::from("audience"))
      .set_jti(String::from("jti"))
      .set_not_before(Utc::now())
      .set_subject(String::from("test"))
      .set_claim(String::from("claim"), json!(String::from("data")))
      .set_footer(String::from("footer"))
      .build()
      .expect("Failed to construct paseto token w/ builder!");

    let decrypted_token = V2Decrypt(
      token,
      Some(String::from("footer")),
      &mut Vec::from("YELLOW SUBMARINE, BLACK WIZARDRY".as_bytes()),
    ).expect("Failed to decrypt token constructed with builder!");

    let parsed: JsonValue = ParseJson(&decrypted_token).expect("Failed to parse finalized token as json!");

    assert!(parsed.get("iat").is_some());
    assert!(parsed.get("iss").is_some());
    assert!(parsed.get("aud").is_some());
    assert!(parsed.get("jti").is_some());
    assert!(parsed.get("sub").is_some());
    assert!(parsed.get("claim").is_some());
    assert!(parsed.get("nbf").is_some());
  }
}
