use std::marker::PhantomData;

use crate::traits::{KVQBinaryStore, KVQSerializable, KVQStoreAdapter, KVQPair};



pub struct KVQStandardAdapter<S: KVQBinaryStore, K: KVQSerializable, V: KVQSerializable> {
  _s: PhantomData<S>,
  _k: PhantomData<K>,
  _v: PhantomData<V>,
}

impl<S: KVQBinaryStore, K: KVQSerializable, V: KVQSerializable> KVQStoreAdapter<S, K, V>
  for KVQStandardAdapter<S,K,V>
{
  fn get_exact(s: &S, key: &K) -> anyhow::Result<V> {
      let r = s.get_exact(&key.to_bytes())?;
      Ok(V::from_bytes(&r))
  }

  fn set_ref(s: &mut S, key: &K, value: &V) -> anyhow::Result<()> {
      s.set(key.to_bytes(), value.to_bytes())
  }
  fn set(s: &mut S, key: K, value: V) -> anyhow::Result<()> {
      s.set(key.to_bytes(), value.to_bytes())
  }

  fn set_many_ref<'a>(s: &mut S, items: &[KVQPair<&'a K, &'a V>]) -> anyhow::Result<()> {
      let pairs: Vec<KVQPair<Vec<u8>, Vec<u8>>> = items
          .iter()
          .map(|kv| KVQPair {
              key: kv.key.to_bytes(),
              value: kv.value.to_bytes(),
          })
          .collect();
      s.set_many_vec(pairs)
  }

  fn set_many(s: &mut S, items: &[KVQPair<K, V>]) -> anyhow::Result<()> {
      let pairs: Vec<KVQPair<Vec<u8>, Vec<u8>>> = items
          .iter()
          .map(|kv| KVQPair {
              key: kv.key.to_bytes(),
              value: kv.value.to_bytes(),
          })
          .collect();
      s.set_many_vec(pairs)
  }

  fn delete(s: &mut S, key: &K) -> anyhow::Result<bool> {
      s.delete(&key.to_bytes())
  }

  fn get_leq_kv(s: &S, key: &K, fuzzy_bytes: usize) -> anyhow::Result<Option<KVQPair<K, V>>> {
      let r = s.get_leq_kv(&key.to_bytes(), fuzzy_bytes)?;
      match r {
          Some(kv) => Ok(Some(KVQPair {
              key: K::from_bytes(&kv.key),
              value: V::from_bytes(&kv.value),
          })),
          None => Ok(None),
      }
  }

  fn get_many_exact(s: &S, keys: &[K]) -> anyhow::Result<Vec<V>> {
      let keys_bytes = keys.iter().map(|k| k.to_bytes()).collect::<Vec<Vec<u8>>>();
      let values_bytes = s.get_many_exact(&keys_bytes)?;
      let values = values_bytes.iter().map(|r| V::from_bytes(r)).collect::<Vec<V>>();
      Ok(values)
  }

  fn delete_many(s: &mut S, keys: &[K]) -> anyhow::Result<Vec<bool>> {
      let mut results: Vec<bool> = Vec::with_capacity(keys.len());

      for k in keys {
          let r = s.delete(&k.to_bytes())?;
          results.push(r)
      }
      Ok(results)
  }

  fn get_leq(s: &S, key: &K, fuzzy_bytes: usize)->anyhow::Result<Option<V>> {
      let r = s.get_leq(&key.to_bytes(), fuzzy_bytes)?;
      match r {
          Some(v) => Ok(Some(V::from_bytes(&v))),
          None => Ok(None),
      }
  }

  fn get_many_leq(s: &S, keys: &[K], fuzzy_bytes: usize)->anyhow::Result<Vec<Option<V>>> {
      let keys_bytes = keys.iter().map(|k| k.to_bytes()).collect::<Vec<Vec<u8>>>();
      let values_bytes = s.get_many_leq(&keys_bytes, fuzzy_bytes)?;
      let values = values_bytes.iter().map(|r| match r {
          Some(v) => Some(V::from_bytes(v)),
          None => None,
      }).collect::<Vec<Option<V>>>();
      Ok(values)
  }

  fn get_many_leq_kv(s: &S, keys: &[K], fuzzy_bytes: usize) -> anyhow::Result<Vec<Option<KVQPair<K,V>>>> {
    let keys_bytes = keys.iter().map(|k| k.to_bytes()).collect::<Vec<Vec<u8>>>();
    let kvs_bytes = s.get_many_leq_kv(&keys_bytes, fuzzy_bytes)?;
      let kvs:  Vec<Option<KVQPair<K, V>>> = kvs_bytes.iter().map(|r| match r {
          Some(kv) => Some(KVQPair {
              key: K::from_bytes(&kv.key),
              value: V::from_bytes(&kv.value),
          }),
          None => None,
      }).collect();
      Ok(kvs)
  }
}
