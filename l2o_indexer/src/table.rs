use l2o_macros::define_table;
use l2o_ord_store::entry::HeaderValue;
use redb::TableDefinition;

define_table! { HEIGHT_TO_BLOCK_HEADER, u32, &HeaderValue }
