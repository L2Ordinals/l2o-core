use std::path::Path;

use kvq::traits::KVQBinaryStore;
use kvq::traits::KVQPair;
use redb::Database;
use redb::ReadableTable;
use redb::TableDefinition;

const TABLE: TableDefinition<&'static [u8], &'static [u8]> = TableDefinition::new("kv");

pub struct KVQReDBStore {
    db: Database,
}
impl KVQReDBStore {
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        Ok(Self {
            db: Database::create(path)?,
        })
    }
}

impl KVQBinaryStore for KVQReDBStore {
    fn get_exact(&self, key: &Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let rxn = self.db.begin_read()?;
        let table = rxn.open_table(TABLE)?;
        let res = table
            .get(key.as_slice())?
            .ok_or(anyhow::anyhow!("Key not found"))?
            .value()
            .to_vec();
        Ok(res)
    }

    fn get_many_exact(&self, keys: &[Vec<u8>]) -> anyhow::Result<Vec<Vec<u8>>> {
        let mut result = Vec::new();
        for key in keys {
            let r = self.get_exact(key)?;
            result.push(r);
        }
        Ok(result)
    }

    fn set(&mut self, key: Vec<u8>, value: Vec<u8>) -> anyhow::Result<()> {
        self.set_ref(&key, &value)
    }

    fn set_ref(&mut self, key: &Vec<u8>, value: &Vec<u8>) -> anyhow::Result<()> {
        let wxn = self.db.begin_write()?;
        {
            let mut table = wxn.open_table(TABLE)?;
            table.insert(key.as_slice(), value.as_slice())?;
        }
        wxn.commit()?;
        Ok(())
    }

    fn set_many_ref<'a>(
        &mut self,
        items: &[KVQPair<&'a Vec<u8>, &'a Vec<u8>>],
    ) -> anyhow::Result<()> {
        let wxn = self.db.begin_write()?;
        {
            let mut table = wxn.open_table(TABLE)?;
            for item in items {
                table.insert(item.key.as_slice(), item.value.as_slice())?;
            }
        }
        Ok(wxn.commit()?)
    }

    fn set_many_vec(&mut self, items: Vec<KVQPair<Vec<u8>, Vec<u8>>>) -> anyhow::Result<()> {
        self.set_many_ref(
            items
                .iter()
                .map(|x| KVQPair {
                    key: &x.key,
                    value: &x.value,
                })
                .collect::<Vec<_>>()
                .as_slice(),
        )
    }

    fn delete(&mut self, key: &Vec<u8>) -> anyhow::Result<bool> {
        let wxn = self.db.begin_write()?;
        let res = {
            let mut table = wxn.open_table(TABLE)?;

            let res = table.remove(key.as_slice())?.is_some();
            res
        };

        wxn.commit()?;
        Ok(res)
    }

    fn delete_many(&mut self, keys: &[Vec<u8>]) -> anyhow::Result<Vec<bool>> {
        let wxn = self.db.begin_write()?;
        let result = {
            let mut table = wxn.open_table(TABLE)?;

            let mut result = Vec::with_capacity(keys.len());
            for key in keys {
                let r = table.remove(key.as_slice())?;
                result.push(r.is_some());
            }
            result
        };
        wxn.commit()?;
        Ok(result)
    }

    fn get_leq(&self, key: &Vec<u8>, fuzzy_bytes: usize) -> anyhow::Result<Option<Vec<u8>>> {
        let rxn = self.db.begin_read()?;
        let table = rxn.open_table(TABLE)?;
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

        let rq = table
            .range(base_key.as_slice()..key_end.as_slice())?
            .next_back();

        match rq {
            Some(Ok((_, v))) => Ok(Some(v.value().to_vec())),
            _ => Ok(None),
        }
    }

    fn get_leq_kv(
        &self,
        key: &Vec<u8>,
        fuzzy_bytes: usize,
    ) -> anyhow::Result<Option<KVQPair<Vec<u8>, Vec<u8>>>> {
        let rxn = self.db.begin_read()?;
        let table = rxn.open_table(TABLE)?;
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

        let rq = table
            .range(base_key.as_slice()..key_end.as_slice())?
            .next_back();

        match rq {
            Some(Ok((k, v))) => Ok(Some(KVQPair {
                key: k.value().to_vec(),
                value: v.value().to_vec(),
            })),
            _ => Ok(None),
        }
    }

    fn get_many_leq(
        &self,
        keys: &[Vec<u8>],
        fuzzy_bytes: usize,
    ) -> anyhow::Result<Vec<Option<Vec<u8>>>> {
        let mut results: Vec<Option<Vec<u8>>> = Vec::with_capacity(keys.len());
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
    ) -> anyhow::Result<Vec<Option<KVQPair<Vec<u8>, Vec<u8>>>>> {
        let mut results: Vec<Option<KVQPair<Vec<u8>, Vec<u8>>>> = Vec::with_capacity(keys.len());
        for k in keys {
            let r = self.get_leq_kv(k, fuzzy_bytes)?;
            results.push(r);
        }
        Ok(results)
    }
}
