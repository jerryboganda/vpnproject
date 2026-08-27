//! Windows Routing, DNS Management, Recovery Journal & WFP Kill Switch

pub mod journal;
pub mod routes;
pub mod wfp;

pub use journal::{RecoveryJournal, RouteSnapshot};
pub use routes::WindowsRouteManager;
pub use wfp::{KillSwitchState, WindowsFirewallManager};
