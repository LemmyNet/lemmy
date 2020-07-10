export enum UserOperation {
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

export enum CommentSortType {
  Hot,
  Top,
  New,
  Old,
}

export enum ListingType {
  All,
  Subscribed,
  Community,
}

export enum DataType {
  Post,
  Comment,
}

export enum SortType {
  Hot,
  New,
  TopDay,
  TopWeek,
  TopMonth,
  TopYear,
  TopAll,
}

export enum SearchType {
  All,
  Comments,
  Posts,
  Communities,
  Users,
  Url,
}

export interface User {
  id: number;
  iss: string;
  username: string;
  show_nsfw: boolean;
  theme: string;
  default_sort_type: SortType;
  default_listing_type: ListingType;
  lang: string;
  avatar?: string;
  show_avatars: boolean;
  unreadCount?: number;
}

export interface UserView {
  id: number;
  actor_id: string;
  name: string;
  avatar?: string;
  email?: string;
  matrix_user_id?: string;
  bio?: string;
  local: boolean;
  published: string;
  number_of_posts: number;
  post_score: number;
  number_of_comments: number;
  comment_score: number;
  banned: boolean;
  show_avatars: boolean;
  send_notifications_to_email: boolean;
}

export interface CommunityUser {
  id: number;
  user_id: number;
  user_actor_id: string;
  user_local: boolean;
  user_name: string;
  avatar?: string;
  community_id: number;
  community_actor_id: string;
  community_local: boolean;
  community_name: string;
  published: string;
}

export interface Community {
  id: number;
  actor_id: string;
  local: boolean;
  name: string;
  title: string;
  description?: string;
  category_id: number;
  creator_id: number;
  removed: boolean;
  deleted: boolean;
  nsfw: boolean;
  published: string;
  updated?: string;
  creator_actor_id: string;
  creator_local: boolean;
  last_refreshed_at: string;
  creator_name: string;
  creator_avatar?: string;
  category_name: string;
  number_of_subscribers: number;
  number_of_posts: number;
  number_of_comments: number;
  user_id?: number;
  subscribed?: boolean;
}

export interface Post {
  id: number;
  name: string;
  url?: string;
  body?: string;
  creator_id: number;
  community_id: number;
  removed: boolean;
  deleted: boolean;
  locked: boolean;
  stickied: boolean;
  embed_title?: string;
  embed_description?: string;
  embed_html?: string;
  thumbnail_url?: string;
  ap_id: string;
  local: boolean;
  nsfw: boolean;
  banned: boolean;
  banned_from_community: boolean;
  published: string;
  updated?: string;
  creator_actor_id: string;
  creator_local: boolean;
  creator_name: string;
  creator_published: string;
  creator_avatar?: string;
  community_actor_id: string;
  community_local: boolean;
  community_name: string;
  community_removed: boolean;
  community_deleted: boolean;
  community_nsfw: boolean;
  number_of_comments: number;
  score: number;
  upvotes: number;
  downvotes: number;
  hot_rank: number;
  newest_activity_time: string;
  user_id?: number;
  my_vote?: number;
  subscribed?: boolean;
  read?: boolean;
  saved?: boolean;
  duplicates?: Array<Post>;
}

export interface Comment {
  id: number;
  ap_id: string;
  local: boolean;
  creator_id: number;
  post_id: number;
  parent_id?: number;
  content: string;
  removed: boolean;
  deleted: boolean;
  read: boolean;
  published: string;
  updated?: string;
  community_id: number;
  community_actor_id: string;
  community_local: boolean;
  community_name: string;
  banned: boolean;
  banned_from_community: boolean;
  creator_actor_id: string;
  creator_local: boolean;
  creator_name: string;
  creator_avatar?: string;
  creator_published: string;
  score: number;
  upvotes: number;
  downvotes: number;
  hot_rank: number;
  user_id?: number;
  my_vote?: number;
  subscribed?: number;
  saved?: boolean;
  user_mention_id?: number; // For mention type
  recipient_id?: number;
  recipient_actor_id?: string;
  recipient_local?: boolean;
  depth?: number;
}

export interface Category {
  id: number;
  name: string;
}

export interface Site {
  id: number;
  name: string;
  description?: string;
  creator_id: number;
  published: string;
  updated?: string;
  creator_name: string;
  number_of_users: number;
  number_of_posts: number;
  number_of_comments: number;
  number_of_communities: number;
  enable_downvotes: boolean;
  open_registration: boolean;
  enable_nsfw: boolean;
}

export interface PrivateMessage {
  id: number;
  creator_id: number;
  recipient_id: number;
  content: string;
  deleted: boolean;
  read: boolean;
  published: string;
  updated?: string;
  ap_id: string;
  local: boolean;
  creator_name: string;
  creator_avatar?: string;
  creator_actor_id: string;
  creator_local: boolean;
  recipient_name: string;
  recipient_avatar?: string;
  recipient_actor_id: string;
  recipient_local: boolean;
}

export enum BanType {
  Community,
  Site,
}

export interface FollowCommunityForm {
  community_id: number;
  follow: boolean;
  auth?: string;
}

export interface GetFollowedCommunitiesForm {
  auth: string;
}

export interface GetFollowedCommunitiesResponse {
  communities: Array<CommunityUser>;
}

export interface GetUserDetailsForm {
  user_id?: number;
  username?: string;
  sort: string;
  page?: number;
  limit?: number;
  community_id?: number;
  saved_only: boolean;
}

export interface UserDetailsResponse {
  user: UserView;
  follows: Array<CommunityUser>;
  moderates: Array<CommunityUser>;
  comments: Array<Comment>;
  posts: Array<Post>;
  admins: Array<UserView>;
}

export interface GetRepliesForm {
  sort: string;
  page?: number;
  limit?: number;
  unread_only: boolean;
  auth?: string;
}

export interface GetRepliesResponse {
  replies: Array<Comment>;
}

export interface GetUserMentionsForm {
  sort: string;
  page?: number;
  limit?: number;
  unread_only: boolean;
  auth?: string;
}

export interface GetUserMentionsResponse {
  mentions: Array<Comment>;
}

export interface EditUserMentionForm {
  user_mention_id: number;
  read?: boolean;
  auth?: string;
}

export interface UserMentionResponse {
  mention: Comment;
}

export interface BanFromCommunityForm {
  community_id: number;
  user_id: number;
  ban: boolean;
  reason?: string;
  expires?: number;
  auth?: string;
}

export interface BanFromCommunityResponse {
  user: UserView;
  banned: boolean;
}

export interface AddModToCommunityForm {
  community_id: number;
  user_id: number;
  added: boolean;
  auth?: string;
}

export interface TransferCommunityForm {
  community_id: number;
  user_id: number;
  auth?: string;
}

export interface TransferSiteForm {
  user_id: number;
  auth?: string;
}

export interface AddModToCommunityResponse {
  moderators: Array<CommunityUser>;
}

export interface GetModlogForm {
  mod_user_id?: number;
  community_id?: number;
  page?: number;
  limit?: number;
}

export interface GetModlogResponse {
  removed_posts: Array<ModRemovePost>;
  locked_posts: Array<ModLockPost>;
  stickied_posts: Array<ModStickyPost>;
  removed_comments: Array<ModRemoveComment>;
  removed_communities: Array<ModRemoveCommunity>;
  banned_from_community: Array<ModBanFromCommunity>;
  banned: Array<ModBan>;
  added_to_community: Array<ModAddCommunity>;
  added: Array<ModAdd>;
}

export interface ModRemovePost {
  id: number;
  mod_user_id: number;
  post_id: number;
  reason?: string;
  removed?: boolean;
  when_: string;
  mod_user_name: string;
  post_name: string;
  community_id: number;
  community_name: string;
}

export interface ModLockPost {
  id: number;
  mod_user_id: number;
  post_id: number;
  locked?: boolean;
  when_: string;
  mod_user_name: string;
  post_name: string;
  community_id: number;
  community_name: string;
}

export interface ModStickyPost {
  id: number;
  mod_user_id: number;
  post_id: number;
  stickied?: boolean;
  when_: string;
  mod_user_name: string;
  post_name: string;
  community_id: number;
  community_name: string;
}

export interface ModRemoveComment {
  id: number;
  mod_user_id: number;
  comment_id: number;
  reason?: string;
  removed?: boolean;
  when_: string;
  mod_user_name: string;
  comment_user_id: number;
  comment_user_name: string;
  comment_content: string;
  post_id: number;
  post_name: string;
  community_id: number;
  community_name: string;
}

export interface ModRemoveCommunity {
  id: number;
  mod_user_id: number;
  community_id: number;
  reason?: string;
  removed?: boolean;
  expires?: number;
  when_: string;
  mod_user_name: string;
  community_name: string;
}

export interface ModBanFromCommunity {
  id: number;
  mod_user_id: number;
  other_user_id: number;
  community_id: number;
  reason?: string;
  banned?: boolean;
  expires?: number;
  when_: string;
  mod_user_name: string;
  other_user_name: string;
  community_name: string;
}

export interface ModBan {
  id: number;
  mod_user_id: number;
  other_user_id: number;
  reason?: string;
  banned?: boolean;
  expires?: number;
  when_: string;
  mod_user_name: string;
  other_user_name: string;
}

export interface ModAddCommunity {
  id: number;
  mod_user_id: number;
  other_user_id: number;
  community_id: number;
  removed?: boolean;
  when_: string;
  mod_user_name: string;
  other_user_name: string;
  community_name: string;
}

export interface ModAdd {
  id: number;
  mod_user_id: number;
  other_user_id: number;
  removed?: boolean;
  when_: string;
  mod_user_name: string;
  other_user_name: string;
}

export interface LoginForm {
  username_or_email: string;
  password: string;
}

export interface RegisterForm {
  username: string;
  email?: string;
  password: string;
  password_verify: string;
  admin: boolean;
  show_nsfw: boolean;
}

export interface LoginResponse {
  jwt: string;
}

export interface UserSettingsForm {
  show_nsfw: boolean;
  theme: string;
  default_sort_type: SortType;
  default_listing_type: ListingType;
  lang: string;
  avatar?: string;
  email?: string;
  matrix_user_id?: string;
  new_password?: string;
  new_password_verify?: string;
  old_password?: string;
  show_avatars: boolean;
  send_notifications_to_email: boolean;
  auth: string;
}

export interface CommunityForm {
  name: string;
  title: string;
  description?: string;
  category_id: number;
  edit_id?: number;
  removed?: boolean;
  deleted?: boolean;
  nsfw: boolean;
  reason?: string;
  expires?: number;
  auth?: string;
}

export interface GetCommunityForm {
  id?: number;
  name?: string;
  auth?: string;
}

export interface GetCommunityResponse {
  community: Community;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  online: number;
}

export interface CommunityResponse {
  community: Community;
}

export interface ListCommunitiesForm {
  sort: string;
  page?: number;
  limit?: number;
  auth?: string;
}

export interface ListCommunitiesResponse {
  communities: Array<Community>;
}

export interface ListCategoriesResponse {
  categories: Array<Category>;
}

export interface PostForm {
  name: string;
  url?: string;
  body?: string;
  community_id: number;
  updated?: number;
  edit_id?: number;
  creator_id: number;
  removed?: boolean;
  deleted?: boolean;
  nsfw: boolean;
  locked?: boolean;
  stickied?: boolean;
  reason?: string;
  auth: string;
}

export interface PostFormParams {
  name: string;
  url?: string;
  body?: string;
  community?: string;
}

export interface GetPostForm {
  id: number;
  auth?: string;
}

export interface GetPostResponse {
  post: Post;
  comments: Array<Comment>;
  community: Community;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
  online: number;
}

export interface SavePostForm {
  post_id: number;
  save: boolean;
  auth?: string;
}

export interface PostResponse {
  post: Post;
}

export interface CommentForm {
  content: string;
  post_id: number;
  parent_id?: number;
  edit_id?: number;
  creator_id?: number;
  removed?: boolean;
  deleted?: boolean;
  reason?: string;
  read?: boolean;
  auth: string;
}

export interface SaveCommentForm {
  comment_id: number;
  save: boolean;
  auth?: string;
}

export interface CommentResponse {
  comment: Comment;
  recipient_ids: Array<number>;
}

export interface CommentLikeForm {
  comment_id: number;
  post_id: number;
  score: number;
  auth?: string;
}

export interface CommentNode {
  comment: Comment;
  children?: Array<CommentNode>;
}

export interface GetPostsForm {
  type_: string;
  sort: string;
  page?: number;
  limit?: number;
  community_id?: number;
  auth?: string;
}

export interface GetPostsResponse {
  posts: Array<Post>;
}

export interface GetCommentsForm {
  type_: string;
  sort: string;
  page?: number;
  limit: number;
  community_id?: number;
  auth?: string;
}

export interface GetCommentsResponse {
  comments: Array<Comment>;
}

export interface CreatePostLikeForm {
  post_id: number;
  score: number;
  auth?: string;
}

export interface SiteForm {
  name: string;
  description?: string;
  enable_downvotes: boolean;
  open_registration: boolean;
  enable_nsfw: boolean;
  auth?: string;
}

export interface GetSiteConfig {
  auth?: string;
}

export interface GetSiteConfigResponse {
  config_hjson: string;
}

export interface SiteConfigForm {
  config_hjson: string;
  auth?: string;
}

export interface GetSiteResponse {
  site: Site;
  admins: Array<UserView>;
  banned: Array<UserView>;
  online: number;
}

export interface SiteResponse {
  site: Site;
}

export interface BanUserForm {
  user_id: number;
  ban: boolean;
  reason?: string;
  expires?: number;
  auth?: string;
}

export interface BanUserResponse {
  user: UserView;
  banned: boolean;
}

export interface AddAdminForm {
  user_id: number;
  added: boolean;
  auth?: string;
}

export interface AddAdminResponse {
  admins: Array<UserView>;
}

export interface SearchForm {
  q: string;
  type_: string;
  community_id?: number;
  sort: string;
  page?: number;
  limit?: number;
  auth?: string;
}

export interface SearchResponse {
  type_: string;
  posts?: Array<Post>;
  comments?: Array<Comment>;
  communities: Array<Community>;
  users: Array<UserView>;
}

export interface DeleteAccountForm {
  password: string;
}

export interface PasswordResetForm {
  email: string;
}

// export interface PasswordResetResponse {
// }

export interface PasswordChangeForm {
  token: string;
  password: string;
  password_verify: string;
}

export interface PrivateMessageForm {
  content: string;
  recipient_id: number;
  auth?: string;
}

export interface PrivateMessageFormParams {
  recipient_id: number;
}

export interface EditPrivateMessageForm {
  edit_id: number;
  content?: string;
  deleted?: boolean;
  read?: boolean;
  auth?: string;
}

export interface GetPrivateMessagesForm {
  unread_only: boolean;
  page?: number;
  limit?: number;
  auth?: string;
}

export interface PrivateMessagesResponse {
  messages: Array<PrivateMessage>;
}

export interface PrivateMessageResponse {
  message: PrivateMessage;
}

export interface UserJoinForm {
  auth: string;
}

export interface UserJoinResponse {
  user_id: number;
}

export type MessageType =
  | EditPrivateMessageForm
  | LoginForm
  | RegisterForm
  | CommunityForm
  | FollowCommunityForm
  | ListCommunitiesForm
  | GetFollowedCommunitiesForm
  | PostForm
  | GetPostForm
  | GetPostsForm
  | GetCommunityForm
  | CommentForm
  | CommentLikeForm
  | SaveCommentForm
  | CreatePostLikeForm
  | BanFromCommunityForm
  | AddAdminForm
  | AddModToCommunityForm
  | TransferCommunityForm
  | TransferSiteForm
  | SaveCommentForm
  | BanUserForm
  | AddAdminForm
  | GetUserDetailsForm
  | GetRepliesForm
  | GetUserMentionsForm
  | EditUserMentionForm
  | GetModlogForm
  | SiteForm
  | SearchForm
  | UserSettingsForm
  | DeleteAccountForm
  | PasswordResetForm
  | PasswordChangeForm
  | PrivateMessageForm
  | EditPrivateMessageForm
  | GetPrivateMessagesForm
  | SiteConfigForm;

type ResponseType =
  | SiteResponse
  | GetFollowedCommunitiesResponse
  | ListCommunitiesResponse
  | GetPostsResponse
  | PostResponse
  | GetRepliesResponse
  | GetUserMentionsResponse
  | ListCategoriesResponse
  | CommunityResponse
  | CommentResponse
  | UserMentionResponse
  | LoginResponse
  | GetModlogResponse
  | SearchResponse
  | BanFromCommunityResponse
  | AddModToCommunityResponse
  | BanUserResponse
  | AddAdminResponse
  | PrivateMessageResponse
  | PrivateMessagesResponse
  | GetSiteConfigResponse;

export interface WebSocketResponse {
  op: UserOperation;
  data: ResponseType;
}

export interface WebSocketJsonResponse {
  op?: string;
  data?: ResponseType;
  error?: string;
  reconnect?: boolean;
}
