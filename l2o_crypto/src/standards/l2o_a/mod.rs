use l2o_common::standards::l2o_a::actions::{deploy::L2ODeployInscription, block::L2OBlockInscription};

use self::proof::L2OAVerifierData;

pub mod proof;


pub type L2ODeployInscriptionV1 = L2ODeployInscription<L2OAVerifierData>;
pub type L2OBlockInscriptionV1 = L2OBlockInscription<L2OAVerifierData>;