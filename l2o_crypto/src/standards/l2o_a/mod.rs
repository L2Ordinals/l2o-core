use l2o_common::standards::l2o_a::actions::block::L2OABlockInscription;
use l2o_common::standards::l2o_a::actions::deploy::L2OADeployInscription;

use self::proof::L2OAVerifierData;
use crate::standards::l2o_a::proof::L2OAProofData;

pub mod proof;

pub type L2OADeployInscriptionV1 = L2OADeployInscription<L2OAVerifierData>;
pub type L2OABlockInscriptionV1 = L2OABlockInscription<L2OAProofData>;
