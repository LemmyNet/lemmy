use lemmy_db::{
    comment_report::CommentReportView,
    post_report::PostReportView,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CreateCommentReport {
    pub comment: i32,
    pub reason: Option<String>,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CommentReportResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize)]
pub struct CreatePostReport {
    pub post: i32,
    pub reason: Option<String>,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostReportResponse {
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommentReports {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub community: i32,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListCommentReportResponse {
    pub reports: Vec<CommentReportView>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPostReports {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub community: i32,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListPostReportResponse {
    pub reports: Vec<PostReportView>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetReportCount {
    pub community: i32,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetReportCountResponse {
    pub community: i32,
    pub comment_reports: usize,
    pub post_reports: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolveCommentReport {
    pub report: uuid::Uuid,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolveCommentReportResponse {
    pub report: uuid::Uuid,
    pub resolved: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvePostReport {
    pub report: uuid::Uuid,
    pub auth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvePostReportResponse {
    pub report: uuid::Uuid,
    pub resolved: bool,
}
