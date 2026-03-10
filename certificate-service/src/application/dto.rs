//! Data contracts for API responses (paginated list, dashboard metrics).
//! Serialize to the JSON shapes expected by the frontend.

use serde::Serialize;

/// Paginated list response: `{ "items": T[], "total": n, "page": p, "limit": l, "total_pages": tp }`
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total: u64, page: u32, limit: u32) -> Self {
        let total_pages = if limit == 0 {
            0
        } else {
            ((total as f64) / (limit as f64)).ceil() as u32
        };
        Self {
            items,
            total,
            page,
            limit,
            total_pages,
        }
    }
}

/// Dashboard metrics: `{ "total": n, "expiring_soon": m }`
#[derive(Debug, Clone, Serialize)]
pub struct DashboardMetrics {
    pub total: u64,
    pub expiring_soon: u64,
}

/// Cursor-paginated list response: `{ "data": T[], "next_cursor": optional }`
#[derive(Debug, Clone, Serialize)]
pub struct CursorListResponse<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
}
