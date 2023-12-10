use serde::{Serialize, Serializer, Deserialize, Deserializer};


#[derive(Clone, PartialEq, Hash)]
pub struct L2OCompactPublicKey(pub [u8; 32]);

impl L2OCompactPublicKey {
  pub fn from_hex(s: &str) -> Result<Self, ()> {
    let bytes = hex::decode(s).unwrap();
    assert_eq!(bytes.len(), 32);
    let mut array = [0u8; 32];
    array.copy_from_slice(&bytes);
    Ok(Self(array))
  }
  pub fn to_hex(&self) -> String {
    hex::encode(self.0)
  }
  pub fn is_zero(&self) -> bool {
    self.0.iter().all(|&x| x == 0)
  }
}

impl Serialize for L2OCompactPublicKey {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
  {
    serializer.serialize_str(&self.to_hex())
  }
}

impl<'de> Deserialize<'de> for L2OCompactPublicKey {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: Deserializer<'de>,
  {
    use serde::de::Error;
    String::deserialize(deserializer)
    .and_then(|string|{
      let mut bytes = [0u8; 32];
      let len = string.len();
      if len == 64 {
        let decoded = hex::decode_to_slice(&string, &mut bytes);
        if decoded.is_err() {
          return Err(Error::custom("Invalid public key"))
        }
        Ok(L2OCompactPublicKey(bytes))
      } else if len == 0 || (len == 1 && string.eq("0")) {
        Ok(L2OCompactPublicKey(bytes))
      }else {
        Err(Error::custom("Invalid public key"))
      }
    })
  }
}



#[derive(Clone, PartialEq, Hash)]
pub struct L2OSignature512(pub [u8; 64]);

impl L2OSignature512 {
  pub fn from_hex(s: &str) -> Result<Self, ()> {
    let bytes = hex::decode(s).unwrap();
    assert_eq!(bytes.len(), 64);
    let mut array = [0u8; 64];
    array.copy_from_slice(&bytes);
    Ok(Self(array))
  }
  pub fn to_hex(&self) -> String {
    hex::encode(self.0)
  }
  pub fn is_zero(&self) -> bool {
    self.0.iter().all(|&x| x == 0)
  }
}

impl Serialize for L2OSignature512 {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
      S: Serializer,
  {
    serializer.serialize_str(&self.to_hex())
  }
}

impl<'de> Deserialize<'de> for L2OSignature512 {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
      D: Deserializer<'de>,
  {
    use serde::de::Error;
    String::deserialize(deserializer)
    .and_then(|string|{
      let mut bytes = [0u8; 64];
      let len = string.len();
      if len == 64 {
        let decoded = hex::decode_to_slice(&string, &mut bytes);
        if decoded.is_err() {
          return Err(Error::custom("Invalid public key"))
        }
        Ok(L2OSignature512(bytes))
      } else if len == 0 || (len == 1 && string.eq("0")) {
        Ok(L2OSignature512(bytes))
      }else {
        Err(Error::custom("Invalid public key"))
      }
    })
  }
}
