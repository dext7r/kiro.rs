//! 数据库存储模块
//!
//! 提供凭据的持久化存储抽象层，支持多种后端：
//! - PostgreSQL (通过 `postgres` feature 启用)
//! - 文件存储 (默认，向后兼容)

#[cfg(feature = "postgres")]
mod pg;
mod store;

pub use store::{CredentialRecord, CredentialStore, PaginatedResult};

#[cfg(feature = "postgres")]
pub use pg::PgCredentialStore;
