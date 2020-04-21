pub mod server;

use crate::ConnectionId;
use actix::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use failure::Error;
use log::{error, info};
use rand::{rngs::ThreadRng, Rng};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use server::ChatServer;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;

#[derive(EnumString, ToString, Debug, Clone)]
pub enum UserOperation {
  Login,
  Register,
  CreateCommunity,
  CreatePost,
  ListCommunities,
  ListCategories,
  GetPost,
  GetCommunity,
  CreateComment,
  EditComment,
  SaveComment,
  CreateCommentLike,
  GetPosts,
  CreatePostLike,
  EditPost,
  SavePost,
  EditCommunity,
  FollowCommunity,
  GetFollowedCommunities,
  GetUserDetails,
  GetReplies,
  GetUserMentions,
  EditUserMention,
  GetModlog,
  BanFromCommunity,
  AddModToCommunity,
  CreateSite,
  EditSite,
  GetSite,
  AddAdmin,
  BanUser,
  Search,
  MarkAllAsRead,
  SaveUserSettings,
  TransferCommunity,
  TransferSite,
  DeleteAccount,
  PasswordReset,
  PasswordChange,
  CreatePrivateMessage,
  EditPrivateMessage,
  GetPrivateMessages,
  UserJoin,
  GetComments,
  GetSiteConfig,
  SaveSiteConfig,
}

#[derive(Clone)]
pub struct WebsocketInfo {
  pub chatserver: Addr<ChatServer>,
  pub id: Option<ConnectionId>,
}
