use lemmy_db::{comment_report::CommentReportView, post_report::PostReportView};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateReport {
    pub report_type: String,
    pub entity_id: i32,
    pub reason: String,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CreateReportResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListReports {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub community: i32,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListReportsResponse {
    pub posts: Vec<PostReportView>,
    pub comments: Vec<CommentReportView>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetReportCount {
    pub community: i32,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetReportCountResponse {
    pub community: i32,
    pub comment_reports: usize,
    pub post_reports: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolveReport {
    pub report_type: String,
    pub report_id: i32,
    pub resolved: bool,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ResolveReportResponse {
    pub report_type: String,
    pub report_id: i32,
    pub resolved: bool,
}
