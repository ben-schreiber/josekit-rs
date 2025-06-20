use std::convert::Into;
use std::fmt::Display;
use std::time::{Duration, SystemTime};

use crate::{JoseError, Map, Number, Value};
use anyhow::bail;

#[derive(Debug, Eq, PartialEq, Clone, Default)]
pub struct JwtPayload {
    claims: Map<String, Value>,
}

impl JwtPayload {
    /// Return a new JWT payload
    pub fn new() -> Self {
        Self { claims: Map::new() }
    }

    /// Return the JWT payload from map.
    ///
    /// # Arguments
    ///
    /// * `map` - JWT payload claims.
    pub fn from_map(map: impl Into<Map<String, Value>>) -> Result<Self, JoseError> {
        let map: Map<String, Value> = map.into();
        for (key, value) in &map {
            Self::check_claim(key, value)?;
        }

        Ok(Self { claims: map })
    }

    /// Set a value for issuer payload claim (iss).
    ///
    /// # Arguments
    ///
    /// * `value` - a issuer
    pub fn set_issuer(&mut self, value: impl Into<String>) {
        let value: String = value.into();
        self.claims.insert("iss".to_string(), Value::String(value));
    }

    /// Return the value for issuer payload claim (iss).
    pub fn issuer(&self) -> Option<&str> {
        match self.claims.get("iss") {
            Some(Value::String(val)) => Some(val),
            _ => None,
        }
    }

    /// Set a value for subject payload claim (sub).
    ///
    /// # Arguments
    ///
    /// * `value` - a subject
    pub fn set_subject(&mut self, value: impl Into<String>) {
        let value: String = value.into();
        self.claims.insert("sub".to_string(), Value::String(value));
    }

    /// Return the value for subject payload claim (sub).
    pub fn subject(&self) -> Option<&str> {
        match self.claims.get("sub") {
            Some(Value::String(val)) => Some(val),
            _ => None,
        }
    }

    /// Set values for audience payload claim (aud).
    ///
    /// # Arguments
    ///
    /// * `values` - a list of audiences
    pub fn set_audience(&mut self, values: Vec<impl Into<String>>) {
        let key = "aud".to_string();
        if values.len() == 1 {
            for val in values {
                let val: String = val.into();
                self.claims.insert(key, Value::String(val));
                break;
            }
        } else if values.len() > 1 {
            let mut vec1 = Vec::with_capacity(values.len());
            let mut vec2 = Vec::with_capacity(values.len());
            for val in values {
                let val: String = val.into();
                vec1.push(Value::String(val.clone()));
                vec2.push(val);
            }
            self.claims.insert(key.clone(), Value::Array(vec1));
        }
    }

    /// Return values for audience payload claim (aud).
    pub fn audience(&self) -> Option<Vec<&str>> {
        match self.claims.get("aud") {
            Some(Value::Array(vals)) => {
                let mut vec = Vec::with_capacity(vals.len());
                for val in vals {
                    match val {
                        Value::String(val2) => {
                            vec.push(val2.as_str());
                        }
                        _ => return None,
                    }
                }
                Some(vec)
            }
            Some(Value::String(val)) => Some(vec![val]),
            _ => None,
        }
    }

    #[cfg(feature = "claims-as-floats")]
    /// Set a system time for expires at payload claim (exp).
    ///
    /// # Arguments
    ///
    /// * `value` - A expiration time on or after which the JWT must not be accepted for processing.
    pub fn set_expires_at(&mut self, value: &SystemTime) {
        let key = "exp".to_string();
        let duration = value.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let val = if duration.subsec_nanos() != 0 {
            Value::Number(
                Number::from_f64(
                    value
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs_f64(),
                )
                .unwrap(),
            )
        } else {
            Value::Number(Number::from(
                value
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ))
        };
        self.claims.insert(key.clone(), val);
    }

    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    /// Set a system time for expires at payload claim (exp).
    ///
    /// # Arguments
    ///
    /// * `value` - A expiration time on or after which the JWT must not be accepted for processing.
    pub fn set_expires_at(&mut self, value: &SystemTime) {
        let key = "exp".to_string();
        let val = Number::from(
            value
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        self.claims.insert(key.clone(), Value::Number(val));
    }

    #[cfg(feature = "claims-as-floats")]
    /// Return the system time for expires at payload claim (exp).
    pub fn expires_at(&self) -> Option<SystemTime> {
        match self.claims.get("exp") {
            Some(Value::Number(val)) => match val.as_f64() {
                Some(val) => Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(val)),
                None => None,
            },
            _ => None,
        }
    }

    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    /// Return the system time for expires at payload claim (exp).
    pub fn expires_at(&self) -> Option<SystemTime> {
        match self.claims.get("exp") {
            Some(Value::Number(val)) => match val.as_u64() {
                Some(val) => Some(SystemTime::UNIX_EPOCH + Duration::from_secs(val)),
                None => None,
            },
            _ => None,
        }
    }

    #[cfg(feature = "claims-as-floats")]
    /// Set a system time for not before payload claim (nbf).
    ///
    /// # Arguments
    ///
    /// * `value` - A time before which the JWT must not be accepted for processing.
    pub fn set_not_before(&mut self, value: &SystemTime) {
        let key = "nbf".to_string();
        let val = Number::from_f64(
            value
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        )
        .unwrap();
        self.claims.insert(key.clone(), Value::Number(val));
    }

    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    /// Set a system time for not before payload claim (nbf).
    ///
    /// # Arguments
    ///
    /// * `value` - A time before which the JWT must not be accepted for processing.
    pub fn set_not_before(&mut self, value: &SystemTime) {
        let key = "nbf".to_string();
        let val = Number::from(
            value
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        self.claims.insert(key.clone(), Value::Number(val));
    }

    #[cfg(feature = "claims-as-floats")]
    /// Return the system time for not before payload claim (nbf).
    pub fn not_before(&self) -> Option<SystemTime> {
        match self.claims.get("nbf") {
            Some(Value::Number(val)) => match val.as_f64() {
                Some(val) => Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(val)),
                None => None,
            },
            _ => None,
        }
    }

    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    /// Return the system time for not before payload claim (nbf).
    pub fn not_before(&self) -> Option<SystemTime> {
        match self.claims.get("nbf") {
            Some(Value::Number(val)) => match val.as_u64() {
                Some(val) => Some(SystemTime::UNIX_EPOCH + Duration::from_secs(val)),
                None => None,
            },
            _ => None,
        }
    }

    #[cfg(feature = "claims-as-floats")]
    /// Set a time for issued at payload claim (iat).
    ///
    /// # Arguments
    ///
    /// * `value` - a time at which the JWT was issued.
    pub fn set_issued_at(&mut self, value: &SystemTime) {
        let key = "iat".to_string();
        let val = Number::from_f64(
            value
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64(),
        )
        .unwrap();
        self.claims.insert(key.clone(), Value::Number(val));
    }

    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    /// Set a time for issued at payload claim (iat).
    ///
    /// # Arguments
    ///
    /// * `value` - a time at which the JWT was issued.
    pub fn set_issued_at(&mut self, value: &SystemTime) {
        let key = "iat".to_string();
        let val = Number::from(
            value
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );
        self.claims.insert(key.clone(), Value::Number(val));
    }

    #[cfg(feature = "claims-as-floats")]
    /// Return the time for a issued at payload claim (iat).
    pub fn issued_at(&self) -> Option<SystemTime> {
        match self.claims.get("iat") {
            Some(Value::Number(val)) => match val.as_f64() {
                Some(val) => Some(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(val)),
                None => None,
            },
            _ => None,
        }
    }

    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    /// Return the time for a issued at payload claim (iat).
    pub fn issued_at(&self) -> Option<SystemTime> {
        match self.claims.get("iat") {
            Some(Value::Number(val)) => match val.as_u64() {
                Some(val) => Some(SystemTime::UNIX_EPOCH + Duration::from_secs(val)),
                None => None,
            },
            _ => None,
        }
    }

    /// Set a value for JWT ID payload claim (jti).
    ///
    /// # Arguments
    ///
    /// * `value` - a JWT ID
    pub fn set_jwt_id(&mut self, value: impl Into<String>) {
        let value: String = value.into();
        self.claims.insert("jti".to_string(), Value::String(value));
    }

    /// Return the value for JWT ID payload claim (jti).
    pub fn jwt_id(&self) -> Option<&str> {
        match self.claims.get("jti") {
            Some(Value::String(val)) => Some(val),
            _ => None,
        }
    }

    /// Set a value for payload claim of a specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - a key name of payload claim
    /// * `value` - a typed value of payload claim
    pub fn set_claim(&mut self, key: &str, value: Option<Value>) -> Result<(), JoseError> {
        (|| -> anyhow::Result<()> {
            match value {
                Some(val) => {
                    Self::check_claim(key, &val)?;
                    self.claims.insert(key.to_string(), val);
                }
                None => {
                    self.claims.remove(key);
                }
            }

            Ok(())
        })()
        .map_err(|err| JoseError::InvalidJwtFormat(err))
    }

    /// Return a value for payload claim of a specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - a key name of payload claim
    pub fn claim(&self, key: &str) -> Option<&Value> {
        self.claims.get(key)
    }

    /// Return values for payload claims set
    pub fn claims_set(&self) -> &Map<String, Value> {
        &self.claims
    }

    #[cfg(feature = "claims-as-floats")]
    fn check_claim(key: &str, value: &Value) -> Result<(), JoseError> {
        (|| -> anyhow::Result<()> {
            match key {
                "iss" | "sub" | "jti" => match &value {
                    Value::String(_) => {}
                    _ => bail!("The JWT {} payload claim must be a string.", key),
                },
                "aud" => match &value {
                    Value::String(_) => {}
                    Value::Array(vals) => {
                        for val in vals {
                            match val {
                                Value::String(_) => {}
                                _ => bail!(
                                    "An element of the JWT {} payload claim must be a string.",
                                    key
                                ),
                            }
                        }
                    }
                    _ => bail!("The JWT {} payload claim must be a string or array.", key),
                },
                "exp" | "nbf" | "iat" => match &value {
                    Value::Number(val) if !is_negative(val) => {}
                    Value::Number(val) => bail!(
                        "The JWT {} payload claim must be a 64bit positive integer or floating point: {}",
                        key,
                        val,
                    ),
                    _ => bail!("The JWT {} header claim must be a number.", key),
                },
                _ => {}
            }

            Ok(())
        })()
        .map_err(|err| JoseError::InvalidJwtFormat(err))
    }

    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    fn check_claim(key: &str, value: &Value) -> Result<(), JoseError> {
        (|| -> anyhow::Result<()> {
            match key {
                "iss" | "sub" | "jti" => match &value {
                    Value::String(_) => {}
                    _ => bail!("The JWT {} payload claim must be a string.", key),
                },
                "aud" => match &value {
                    Value::String(_) => {}
                    Value::Array(vals) => {
                        for val in vals {
                            match val {
                                Value::String(_) => {}
                                _ => bail!(
                                    "An element of the JWT {} payload claim must be a string.",
                                    key
                                ),
                            }
                        }
                    }
                    _ => bail!("The JWT {} payload claim must be a string or array.", key),
                },
                "exp" | "nbf" | "iat" => match &value {
                    Value::Number(val) => match val.as_u64() {
                        Some(_) => {}
                        None => bail!(
                            "The JWT {} payload claim must be a positive integer within 64bit.",
                            key
                        ),
                    },
                    _ => bail!("The JWT {} header claim must be a string.", key),
                },
                _ => {}
            }

            Ok(())
        })()
        .map_err(|err| JoseError::InvalidJwtFormat(err))
    }
}

impl AsRef<Map<String, Value>> for JwtPayload {
    fn as_ref(&self) -> &Map<String, Value> {
        &self.claims
    }
}

impl Into<Map<String, Value>> for JwtPayload {
    fn into(self) -> Map<String, Value> {
        self.claims
    }
}

impl Display for JwtPayload {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let val = serde_json::to_string(&self.claims).map_err(|_e| std::fmt::Error {})?;
        fmt.write_str(&val)
    }
}

#[cfg(feature = "claims-as-floats")]
fn is_negative(num: &Number) -> bool {
    if let Some(num) = num.as_i128() {
        num < 0
    } else if let Some(num) = num.as_i64() {
        num < 0
    } else if let Some(num) = num.as_f64() {
        num < 0.0
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    use std::time::SystemTime;
    #[cfg(feature = "claims-as-floats")]
    use std::time::{Duration, SystemTime};

    use anyhow::Result;
    #[cfg(all(feature = "claims-as-ints", not(feature = "claims-as-floats")))]
    use serde_json::json;
    #[cfg(feature = "claims-as-floats")]
    use serde_json::{json, Number, Value};

    use super::JwtPayload;

    #[test]
    fn test_new_payload() -> Result<()> {
        let mut payload = JwtPayload::new();
        payload.set_issuer("iss");
        payload.set_subject("sub");
        payload.set_audience(vec!["aud0", "aud1"]);
        payload.set_expires_at(&SystemTime::UNIX_EPOCH);
        payload.set_not_before(&SystemTime::UNIX_EPOCH);
        payload.set_issued_at(&SystemTime::UNIX_EPOCH);
        payload.set_jwt_id("jti");
        payload.set_claim("payload_claim", Some(json!("payload_claim")))?;

        assert!(matches!(payload.issuer(), Some("iss")));
        assert!(matches!(payload.subject(), Some("sub")));
        assert!(
            matches!(payload.audience(), Some(ref vals) if vals == &vec!["aud0".to_string(), "aud1".to_string()])
        );
        assert!(matches!(payload.expires_at(), Some(ref val) if val == &SystemTime::UNIX_EPOCH));
        assert!(matches!(payload.not_before(), Some(ref val) if val == &SystemTime::UNIX_EPOCH));
        assert!(matches!(payload.issued_at(), Some(ref val) if val == &SystemTime::UNIX_EPOCH));
        assert!(matches!(payload.jwt_id(), Some("jti")));
        assert!(
            matches!(payload.claim("payload_claim"), Some(val) if val == &json!("payload_claim"))
        );

        Ok(())
    }

    #[cfg(feature = "claims-as-floats")]
    #[test]
    fn test_f64_time_payload() -> Result<()> {
        let mut payload = JwtPayload::new();
        payload.set_issuer("iss");
        payload.set_subject("sub");
        payload.set_audience(vec!["aud0", "aud1"]);
        payload.set_expires_at(
            &SystemTime::UNIX_EPOCH
                .checked_add(Duration::new(1, 1))
                .unwrap(),
        );
        payload.set_not_before(
            &SystemTime::UNIX_EPOCH
                .checked_add(Duration::new(12, 12))
                .unwrap(),
        );
        payload.set_issued_at(
            &SystemTime::UNIX_EPOCH
                .checked_add(Duration::new(123, 123))
                .unwrap(),
        );
        payload.set_jwt_id("jti");
        payload.set_claim("payload_claim", Some(json!("payload_claim")))?;

        assert!(matches!(payload.issuer(), Some("iss")));
        assert!(matches!(payload.subject(), Some("sub")));
        assert!(
            matches!(payload.audience(), Some(ref vals) if vals == &vec!["aud0".to_string(), "aud1".to_string()])
        );
        assert!(
            matches!(payload.claim("exp"), Some(Value::Number(ref val)) if val == &Number::from_f64(1.000000001).unwrap())
        );
        assert!(
            matches!(payload.claim("nbf"), Some(Value::Number(ref val)) if val == &Number::from_f64(12.000000012).unwrap())
        );
        assert!(
            matches!(payload.claim("iat"), Some(Value::Number(ref val)) if val == &Number::from_f64(123.000000123).unwrap())
        );
        assert!(
            matches!(payload.expires_at(), Some(ref val) if val == &SystemTime::UNIX_EPOCH.checked_add(Duration::new(1, 1)).unwrap())
        );
        assert!(
            matches!(payload.not_before(), Some(ref val) if val == &SystemTime::UNIX_EPOCH.checked_add(Duration::new(12, 12)).unwrap())
        );
        assert!(
            matches!(payload.issued_at(), Some(ref val) if val == &SystemTime::UNIX_EPOCH.checked_add(Duration::new(123, 123)).unwrap())
        );
        assert!(matches!(payload.jwt_id(), Some("jti")));
        assert!(
            matches!(payload.claim("payload_claim"), Some(val) if val == &json!("payload_claim"))
        );

        Ok(())
    }
}
