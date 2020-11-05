use n3_machine::Query;

use super::base::CandidatesMachine;

pub struct CudaMachine;

impl CandidatesMachine for CudaMachine {
    fn get_candidates() -> Vec<Query> {
        // TODO: detect devices
        vec![Query {
            device: Some("cuda".to_string()),
            id: Some("0".to_string()),
            ..Default::default()
        }]
    }
}