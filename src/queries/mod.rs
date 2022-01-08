use crate::connections::ConnectionService;
use crate::values::payloads::QueryRequest;
use crate::values::payloads::QueryResponse;

pub struct QueryService {

    connection_service: ConnectionService

}


impl QueryService {

    pub fn new(connection_service: ConnectionService) -> QueryService {
        QueryService {
            connection_service
        }
    }

    pub async fn query(&self, request: &QueryRequest) -> QueryResponse {
        self.connection_service.get(&request.db_id);

        QueryResponse {
            column_names: vec![],
            rows: vec![]
        }
    }

}