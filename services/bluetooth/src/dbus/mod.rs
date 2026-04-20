#![allow(clippy::all)]
#![allow(missing_docs)]
#![allow(unused)]

mod adapter1;
mod agent_manager1;
mod battery1;
mod device1;

pub use adapter1::Adapter1Proxy;
pub use agent_manager1::AgentManager1Proxy;
pub use battery1::Battery1Proxy;
pub use device1::Device1Proxy;
