use std::collections::BTreeMap;

use crate::traits::KVQBinaryStore;
use crate::traits::KVQPair;

pub struct KVQSimpleMemoryBackingStore {
    map: BTreeMap<Vec<u8>, Vec<u8>>,
}
impl KVQSimpleMemoryBackingStore {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
}

impl KVQBinaryStore for KVQSimpleMemoryBackingStore {
    fn get_exact(&self, key: &Vec<u8>) -> anyhow::Result<&Vec<u8>> {
        match self.map.get(key) {
            Some(v) => Ok(v),
            None => anyhow::bail!("Key not found"),
        }
    }

    fn get_many_exact(&self, keys: &[Vec<u8>]) -> anyhow::Result<Vec<&Vec<u8>>> {
        let mut result = Vec::new();
        for key in keys {
            let r = self.get_exact(key)?;
            result.push(r);
        }
        Ok(result)
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> anyhow::Result<()> {
        self.map.insert(key, value);
        Ok(())
    }

    fn set_ptr(&mut self, key: &Vec<u8>, value: &Vec<u8>) -> anyhow::Result<()> {
        self.map.insert(key.clone(), value.clone());
        Ok(())
    }

    fn set_many_ref<'a>(
        &mut self,
        items: &[KVQPair<&'a Vec<u8>, &'a Vec<u8>>],
    ) -> anyhow::Result<()> {
        for item in items {
            self.map.insert(item.key.clone(), item.value.clone());
        }
        Ok(())
    }

    fn set_many_vec(&mut self, items: Vec<KVQPair<Vec<u8>, Vec<u8>>>) -> anyhow::Result<()> {
        for item in items {
            self.map.insert(item.key.clone(), item.value.clone());
        }
        Ok(())
    }

    fn delete(&mut self, key: &Vec<u8>) -> anyhow::Result<bool> {
        match self.map.remove(key) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    fn delete_many(&mut self, keys: &[Vec<u8>]) -> anyhow::Result<Vec<bool>> {
        let mut result = Vec::with_capacity(keys.len());
        for key in keys {
            let r = self.delete(key)?;
            result.push(r);
        }
        Ok(result)
    }

    fn get_leq(&self, key: &Vec<u8>, fuzzy_bytes: usize) -> anyhow::Result<Option<&Vec<u8>>> {
        let key_end = key.to_vec();
        let mut base_key = key.to_vec();
        let key_len = base_key.len();
        if fuzzy_bytes > key_len {
            return Err(anyhow::anyhow!(
                "Fuzzy bytes must be less than or equal to key length"
            ));
        }

        for i in 0..fuzzy_bytes {
            base_key[key_len - i - 1] = 0;
        }
        let rq = self.map.range(base_key..key_end).next_back();

        if rq.is_none() {
            Ok(None)
        } else {
            let p = rq.unwrap().1;

            Ok(Some(p))
        }
    }

    fn get_leq_kv(
        &self,
        key: &Vec<u8>,
        fuzzy_bytes: usize,
    ) -> anyhow::Result<Option<KVQPair<&Vec<u8>, &Vec<u8>>>> {
        let key_end = key.to_vec();
        let mut base_key = key.to_vec();
        let key_len = base_key.len();
        if fuzzy_bytes > key_len {
            return Err(anyhow::anyhow!(
                "Fuzzy bytes must be less than or equal to key length"
            ));
        }

        for i in 0..fuzzy_bytes {
            base_key[key_len - i - 1] = 0;
        }
        let rq = self.map.range(base_key..key_end).next_back();

        if rq.is_none() {
            Ok(None)
        } else {
            let p = rq.unwrap();
            Ok(Some(KVQPair {
                key: p.0,
                value: p.1,
            }))
        }
    }

    fn get_many_leq(
        &self,
        keys: &[Vec<u8>],
        fuzzy_bytes: usize,
    ) -> anyhow::Result<Vec<Option<&Vec<u8>>>> {
        let mut results: Vec<Option<&Vec<u8>>> = Vec::with_capacity(keys.len());
        for k in keys {
            let r = self.get_leq(k, fuzzy_bytes)?;
            results.push(r);
        }
        Ok(results)
    }

    fn get_many_leq_kv(
        &self,
        keys: &[Vec<u8>],
        fuzzy_bytes: usize,
    ) -> anyhow::Result<Vec<Option<KVQPair<&Vec<u8>, &Vec<u8>>>>> {
        let mut results: Vec<Option<KVQPair<&Vec<u8>, &Vec<u8>>>> = Vec::with_capacity(keys.len());
        for k in keys {
            let r = self.get_leq_kv(k, fuzzy_bytes)?;
            results.push(r);
        }
        Ok(results)
    }
}
