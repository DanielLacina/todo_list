mod database;
mod rocksdb_engine;
use chrono::Local;
use database::Database;
use proto::todo_list_server::{TodoList, TodoListServer};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tonic::transport::Server;

mod proto {
    tonic::include_proto!("todo_list");
    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("todo_list_descriptor");
}

#[derive(Default)]
struct TodoListService<T: Database> {
    db: Arc<RwLock<T>>,
}

#[tonic::async_trait]
impl<T: Database> TodoList for TodoListService<T> {
    async fn get_event(
        &self,
        request: tonic::Request<proto::TodoListTimestamp>,
    ) -> Result<tonic::Response<proto::TodoListKv>, tonic::Status> {
        let input = request.get_ref();
        let lock = self.db.read().await;
        let event = lock.get(&input.timestamp);
        if let Some(event) = event {
            return Ok(tonic::Response::new(proto::TodoListKv {
                timestamp: input.timestamp.clone(),
                event: String::from_utf8(event).unwrap(),
            }));
        } else {
            return Err(tonic::Status::not_found(format!(
                "Event with timestamp: {} is not found",
                input.timestamp
            )));
        }
    }
    async fn add_event(
        &self,
        request: tonic::Request<proto::TodoListEvent>,
    ) -> Result<tonic::Response<proto::TodoListKv>, tonic::Status> {
        let input = request.get_ref();
        let mut lock = self.db.write().await;
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        lock.insert(&timestamp, &input.event);
        Ok(tonic::Response::new(proto::TodoListKv {
            timestamp,
            event: input.event.clone(),
        }))
    }
    async fn remove_event(
        &self,
        request: tonic::Request<proto::TodoListTimestamp>,
    ) -> Result<tonic::Response<proto::TodoListResponse>, tonic::Status> {
        let input = request.get_ref();
        let mut lock = self.db.write().await;
        lock.delete(&input.timestamp);
        Ok(tonic::Response::new(proto::TodoListResponse {
            status: "Event successfully deleted".to_string(),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let todo_list = TodoListService::<RocksDBEngine>::default();
    let service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::FILE_DESCRIPTOR_SET)
        .build()?;
    Server::builder()
        .add_service(service)
        .add_service(TodoListServer::new(todo_list))
        .serve(addr)
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use database::MockDatabase;

    fn todo_list_service() -> TodoListService<MockDatabase> {
        TodoListService::<MockDatabase>::default()
    }

    fn unwrap_response<T: Clone>(response: Result<tonic::Response<T>, tonic::Status>) -> T {
        response.unwrap().get_ref().clone()
    }

    async fn add_event(
        todo_list_service: &TodoListService<MockDatabase>,
        event: &str,
    ) -> proto::TodoListKv {
        unwrap_response(
            todo_list_service
                .add_event(tonic::Request::new(proto::TodoListEvent {
                    event: event.to_string(),
                }))
                .await,
        )
    }

    async fn get_event(
        todo_list_service: &TodoListService<MockDatabase>,
        timestamp: &str,
    ) -> proto::TodoListKv {
        unwrap_response(
            todo_list_service
                .get_event(tonic::Request::new(proto::TodoListTimestamp {
                    timestamp: timestamp.to_string(),
                }))
                .await,
        )
    }

    #[tokio::test]
    async fn test_add_event() {
        let todo_list_service = todo_list_service();
        let event = String::from("go to school");
        let add_event_response = todo_list_service
            .add_event(tonic::Request::new(proto::TodoListEvent {
                event: event.clone(),
            }))
            .await;
        assert!(add_event_response.is_ok());
        let get_event_response = get_event(
            &todo_list_service,
            &unwrap_response(add_event_response).timestamp,
        )
        .await;
        assert_eq!(get_event_response.event, event);
    }

    #[tokio::test]
    async fn test_remove_event() {
        let todo_list_service = todo_list_service();
        let event = String::from("go to school");
        let event_kv = add_event(&todo_list_service, &event).await;
        let remove_event_response = todo_list_service
            .remove_event(tonic::Request::new(proto::TodoListTimestamp {
                timestamp: event_kv.timestamp.clone(),
            }))
            .await;
        assert!(remove_event_response.is_ok());
        let get_event_response = todo_list_service
            .get_event(tonic::Request::new(proto::TodoListTimestamp {
                timestamp: event_kv.timestamp,
            }))
            .await;
        assert!(get_event_response.is_err())
    }
}
