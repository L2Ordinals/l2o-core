#[macro_export]
macro_rules! set_state {
    ($instance:expr,$checkpoint_id:expr,$pos:expr,$value:expr) => {
        Sha256StateRootTree::<S>::set_leaf(
            &mut $instance,
            &KVQMerkleNodeKey::from_identifier_position_ref(
                &SHA256_STATE_ROOT_TREE_ID,
                $checkpoint_id,
                &$pos,
            ),
            $value,
        )?;
        Blake3StateRootTree::<S>::set_leaf(
            &mut $instance,
            &KVQMerkleNodeKey::from_identifier_position_ref(
                &BLAKE3_STATE_ROOT_TREE_ID,
                $checkpoint_id,
                &$pos,
            ),
            $value,
        )?;
        Keccak256StateRootTree::<S>::set_leaf(
            &mut $instance,
            &KVQMerkleNodeKey::from_identifier_position_ref(
                &KECCAK256_STATE_ROOT_TREE_ID,
                $checkpoint_id,
                &$pos,
            ),
            $value,
        )?;
        PoseidonGoldilocksStateRootTree::<S>::set_leaf(
            &mut $instance,
            &KVQMerkleNodeKey::from_identifier_position_ref(
                &POSEIDONGOLDILOCKS_STATE_ROOT_TREE_ID,
                $checkpoint_id,
                &$pos,
            ),
            hash256_to_goldilocks_hash(&$value),
        )?;
    };
}

#[macro_export]
macro_rules! get_state {
    ($hash:expr,$instance:expr,$checkpoint_id:expr,$pos:expr,$get_fn:tt,$convert_fn:path) => {
        match $hash {
            L2OAHashFunction::Sha256 => Sha256StateRootTree::<S>::$get_fn(
                &mut $instance,
                &KVQMerkleNodeKey::from_identifier_position_ref(
                    &SHA256_STATE_ROOT_TREE_ID,
                    $checkpoint_id,
                    &$pos,
                ),
            ),
            L2OAHashFunction::BLAKE3 => Blake3StateRootTree::<S>::$get_fn(
                &mut $instance,
                &KVQMerkleNodeKey::from_identifier_position_ref(
                    &BLAKE3_STATE_ROOT_TREE_ID,
                    $checkpoint_id,
                    &$pos,
                ),
            ),
            L2OAHashFunction::Keccak256 => Keccak256StateRootTree::<S>::$get_fn(
                &mut $instance,
                &KVQMerkleNodeKey::from_identifier_position_ref(
                    &KECCAK256_STATE_ROOT_TREE_ID,
                    $checkpoint_id,
                    &$pos,
                ),
            ),
            L2OAHashFunction::PoseidonGoldilocks => {
                let p = PoseidonGoldilocksStateRootTree::<S>::$get_fn(
                    &mut $instance,
                    &KVQMerkleNodeKey::from_identifier_position_ref(
                        &POSEIDONGOLDILOCKS_STATE_ROOT_TREE_ID,
                        $checkpoint_id,
                        &$pos,
                    ),
                )?;
                Ok($convert_fn(&p))
            }
        }
    };
}

#[macro_export]
macro_rules! rpc_call {
    ($instance:ident,$param:expr, $rtype:ty) => {{
        let response = $instance
            .client
            .post(&$instance.url)
            .json(&RpcRequest {
                jsonrpc: Version::V2,
                request: $param,
                id: Id::Number(1),
            })
            .send()
            .await?
            .json::<Value>()
            .await?;

        Ok(serde_json::from_value::<$rtype>(
            response["result"].clone(),
        )?)
    }};
}

// https://www.reddit.com/r/rust/comments/17ln23t/change_my_mind_rust_should_use_the_operator_to/
// ¿
#[macro_export]
macro_rules! quick {
    ($fn_result:expr) => {{
        match $fn_result {
            Ok(res) => return Ok(res),
            Err(err) => err,
        }
    }};
}
