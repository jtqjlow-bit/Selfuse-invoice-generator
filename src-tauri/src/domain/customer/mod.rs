pub mod commands;
mod repository;
mod service;
pub mod types;

#[cfg(test)]
mod tests;

pub use service::{archive, create, find_by_id, list, search, unarchive, update};
pub use types::{
    CreateCustomerInput, Customer, CustomerType, UpdateCustomerInput,
};
