export enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, ListCategories, GetPost, GetCommunity, CreateComment, EditComment, SaveComment, CreateCommentLike, GetPosts, CreatePostLike, EditPost, SavePost, EditCommunity, FollowCommunity, GetFollowedCommunities, GetUserDetails, GetReplies, GetModlog, BanFromCommunity, AddModToCommunity, CreateSite, EditSite, GetSite, AddAdmin, BanUser, Search, MarkAllAsRead
}

export enum CommentSortType {
  Hot, Top, New
}

export enum ListingType {
  All, Subscribed, Community
}

export enum SortType {
  Hot, New, TopDay, TopWeek, TopMonth, TopYear, TopAll
}

export enum SearchType {
  Both, Comments, Posts
}

export interface User {
  id: number;
  iss: string;
  username: string;
}

export interface UserView {
  id: number;
  name: string;
  fedi_name: string;
  published: string;
  number_of_posts: number;
  post_score: number;
  number_of_comments: number;
  comment_score: number;
}

export interface CommunityUser {
  id: number;
  user_id: number;
  user_name: string;
  community_id: number;
  community_name: string;
  published: string;
}

export interface Community {
  id: number;
  name: string;
  title: string;
  description?: string;
  category_id: number;
  creator_id: number;
  removed: boolean;
  deleted: boolean;
  published: string;
  updated?: string;
  creator_name: string;
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
  published: string;
  updated?: string;
  creator_name: string;
  community_name: string;
  community_removed: boolean;
  number_of_comments: number;
  score: number;
  upvotes: number;
  downvotes: number;
  hot_rank: number;
  user_id?: number;
  my_vote?: number;
  subscribed?: boolean;
  read?: boolean;
  saved?: boolean;
}

export interface Comment {
  id: number;
  creator_id: number;
  post_id: number,
  parent_id?: number;
  content: string;
  removed: boolean;
  deleted: boolean;
  read: boolean;
  published: string;
  updated?: string;
  community_id: number,
  banned: boolean;
  banned_from_community: boolean;
  creator_name: string;
  score: number;
  upvotes: number;
  downvotes: number;
  user_id?: number;
  my_vote?: number;
  saved?: boolean;
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
}

export interface FollowCommunityForm {
  community_id: number;
  follow: boolean;
  auth?: string;
}

export interface GetFollowedCommunitiesResponse {
  op: string;
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
  op: string;
  user: UserView;
  follows: Array<CommunityUser>;
  moderates: Array<CommunityUser>;
  comments: Array<Comment>;
  posts: Array<Post>;
}

export interface GetRepliesForm {
  sort: string; // TODO figure this one out
  page?: number;
  limit?: number;
  unread_only: boolean;
  auth?: string;
}

export interface GetRepliesResponse {
  op: string;
  replies: Array<Comment>;
}

export interface BanFromCommunityForm {
  community_id: number;
  user_id: number;
  ban: boolean;
  reason?: string,
  expires?: number,
  auth?: string;
}

export interface BanFromCommunityResponse {
  op: string;
  user: UserView,
  banned: boolean,
}

export interface AddModToCommunityForm {
  community_id: number;
  user_id: number;
  added: boolean;
  auth?: string;
}

export interface AddModToCommunityResponse {
  op: string;
  moderators: Array<CommunityUser>;
}

export interface GetModlogForm {
  mod_user_id?: number;
  community_id?: number;
  page?: number;
  limit?: number;
}

export interface GetModlogResponse {
  op: string;
  removed_posts: Array<ModRemovePost>,
  locked_posts: Array<ModLockPost>,
  removed_comments: Array<ModRemoveComment>,
  removed_communities: Array<ModRemoveCommunity>,
  banned_from_community: Array<ModBanFromCommunity>,
  banned: Array<ModBan>,
  added_to_community: Array<ModAddCommunity>,
  added: Array<ModAdd>,
}

export interface ModRemovePost {
    id: number;
    mod_user_id: number;
    post_id: number;
    reason?: string;
    removed?: boolean;
    when_: string
    mod_user_name: string;
    post_name: string;
    community_id: number;
    community_name: string;
}

export interface ModLockPost {
  id: number,
  mod_user_id: number,
  post_id: number,
  locked?: boolean,
  when_: string,
  mod_user_name: string,
  post_name: string,
  community_id: number,
  community_name: string,
}

export interface ModRemoveComment {
  id: number,
  mod_user_id: number,
  comment_id: number,
  reason?: string,
  removed?: boolean,
  when_: string,
  mod_user_name: string,
  comment_user_id: number,
  comment_user_name: string,
  comment_content: string,
  post_id: number,
  post_name: string,
  community_id: number,
  community_name: string,
}

export interface ModRemoveCommunity {
  id: number,
  mod_user_id: number,
  community_id: number,
  reason?: string,
  removed?: boolean,
  expires?: number,
  when_: string,
  mod_user_name: string,
  community_name: string,
}

export interface ModBanFromCommunity {
  id: number,
  mod_user_id: number,
  other_user_id: number,
  community_id: number,
  reason?: string,
  banned?: boolean,
  expires?: number,
  when_: string,
  mod_user_name: string,
  other_user_name: string,
  community_name: string,
}

export interface ModBan {
  id: number,
  mod_user_id: number,
  other_user_id: number,
  reason?: string,
  banned?: boolean,
  expires?: number,
  when_: string,
  mod_user_name: string,
  other_user_name: string,
}

export interface ModAddCommunity {
  id: number,
  mod_user_id: number,
  other_user_id: number,
  community_id: number,
  removed?: boolean,
  when_: string,
  mod_user_name: string,
  other_user_name: string,
  community_name: string,
}

export interface ModAdd {
  id: number,
  mod_user_id: number,
  other_user_id: number,
  removed?: boolean,
  when_: string,
  mod_user_name: string,
  other_user_name: string,
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
}

export interface LoginResponse {
  op: string;
  jwt: string;
}



export interface CommunityForm {
  name: string;
  title: string;
  description?: string,
  category_id: number,
  edit_id?: number;
  removed?: boolean;
  deleted?: boolean;
  reason?: string;
  expires?: number;
  auth?: string;
}

export interface GetCommunityResponse {
  op: string;
  community: Community;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
}


export interface CommunityResponse {
  op: string;
  community: Community;
}

export interface ListCommunitiesForm {
  sort: string;
  page?: number;
  limit?: number;
  auth?: string;
}

export interface ListCommunitiesResponse {
  op: string;
  communities: Array<Community>;
}

export interface ListCategoriesResponse {
  op: string;
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
  locked?: boolean;
  reason?: string;
  auth: string;
}

export interface GetPostResponse {
  op: string;
  post: Post;
  comments: Array<Comment>;
  community: Community;
  moderators: Array<CommunityUser>;
  admins: Array<UserView>;
}

export interface SavePostForm {
  post_id: number;
  save: boolean;
  auth?: string;
}

export interface PostResponse {
  op: string;
  post: Post;
}

export interface CommentForm {
  content: string;
  post_id: number;
  parent_id?: number;
  edit_id?: number;
  creator_id: number;
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
  op: string;
  comment: Comment;
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
  op: string;
  posts: Array<Post>;
}

export interface CreatePostLikeForm {
  post_id: number;
  score: number;
  auth?: string;
}

export interface CreatePostLikeResponse {
  op: string;
  post: Post;
}

export interface SiteForm {
  name: string;
  description?: string,
  removed?: boolean;
  reason?: string;
  expires?: number;
  auth?: string;
}

export interface GetSiteResponse {
  op: string;
  site: Site;
  admins: Array<UserView>;
  banned: Array<UserView>;
}


export interface SiteResponse {
  op: string;
  site: Site;
}

export interface BanUserForm {
  user_id: number;
  ban: boolean;
  reason?: string,
  expires?: number,
  auth?: string;
}

export interface BanUserResponse {
  op: string;
  user: UserView,
  banned: boolean,
}

export interface AddAdminForm {
  user_id: number;
  added: boolean;
  auth?: string;
}

export interface AddAdminResponse {
  op: string;
  admins: Array<UserView>;
}

export interface SearchForm {
  q: string;
  type_: string;
  community_id?: number;
  sort: string;
  page?: number;
  limit?: number;
}

export interface SearchResponse {
  op: string;
  posts?: Array<Post>;
  comments?: Array<Comment>;
}
