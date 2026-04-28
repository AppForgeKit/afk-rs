use std::collections::HashMap;
use tonic::Status;
use tonic::metadata::{AsciiMetadataKey, AsciiMetadataValue};
use tonic::service::Interceptor;

#[derive(Clone)]
pub struct Metadata(pub Option<HashMap<String, String>>);

impl Interceptor for Metadata {
    fn call(&mut self, request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        match &self.0 {
            None => Ok(request),
            Some(metadata) => {
                if metadata.is_empty() {
                    return Ok(request);
                }

                let mut new_request = request;

                let new_metadata = new_request.metadata_mut();

                for (key, value) in metadata.iter() {
                    if !new_metadata.contains_key(key) {
                        new_metadata.insert(
                            key.parse::<AsciiMetadataKey>()
                                .map_err(|err| Status::internal(err.to_string()))?,
                            value
                                .parse::<AsciiMetadataValue>()
                                .map_err(|err| Status::internal(err.to_string()))?,
                        );
                    }
                }

                Ok(new_request)
            }
        }
    }
}
