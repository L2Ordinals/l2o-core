use l2o_common::standards::l2o_a::actions::block::L2OBlockInscription;
use l2o_common::standards::l2o_a::actions::deploy::L2ODeployInscription;

use self::proof::L2OAVerifierData;
use crate::standards::l2o_a::proof::L2OAProofData;

pub mod proof;

pub type L2ODeployInscriptionV1 = L2ODeployInscription<L2OAVerifierData>;
pub type L2OBlockInscriptionV1 = L2OBlockInscription<L2OAProofData>;
