//! Admin tools for cache management
//!
//! Provides CLI subtools for cache inspection, garbage collection, and health checks.

pub mod cache_gc;
pub mod cache_health;
pub mod cache_stats;

pub use cache_gc::{run_cache_gc, GcOptions, GcReport};
pub use cache_health::{run_cache_health, HealthCheck, HealthReport, HealthStatus};
pub use cache_stats::{run_cache_stats, CacheStatsReport};
