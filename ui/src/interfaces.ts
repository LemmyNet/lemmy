export enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, GetPost, GetCommunity, CreateComment, EditComment, CreateCommentLike, GetPosts, CreatePostLike
}

export interface User {
  id: number;
  iss: string;
  username: string;
}

export interface Community {
  id: number;
  name: string;
  published: string;
  updated?: string;
}

export interface CommunityForm {
  name: string;
  auth?: string;
}

export interface CommunityResponse {
  op: string;
  community: Community;
}

export interface ListCommunitiesResponse {
  op: string;
  communities: Array<Community>;
}

export interface Post {
  user_id?: number;
  my_vote?: number;
  id: number;
  name: string;
  url?: string;
  body?: string;
  creator_id: number;
  creator_name: string;
  community_id: number;
  community_name: string;
  number_of_comments: number;
  score: number;
  upvotes: number;
  downvotes: number;
  hot_rank: number;
  published: string;
  updated?: string;
}

export interface PostForm {
  name: string;
  url?: string;
  body?: string;
  community_id: number;
  updated?: number;
  auth: string;
}

export interface PostResponse {
  op: string;
  post: Post;
  comments: Array<Comment>;
}

export interface Comment {
  id: number;
  content: string;
  creator_id: number;
  post_id: number,
  parent_id?: number;
  published: string;
  updated?: string;
  score: number;
  upvotes: number;
  downvotes: number;
  my_vote?: number;
}

export interface CommentForm {
  content: string;
  post_id: number;
  parent_id?: number;
  edit_id?: number;
  auth: string;
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

export interface CreateCommentLikeResponse {
  op: string;
  comment: Comment;
}

export interface GetPostsForm {
  type_: string;
  sort: string;
  limit: number;
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

export interface LoginForm {
  username_or_email: string;
  password: string;
}

export interface RegisterForm {
  username: string;
  email?: string;
  password: string;
  password_verify: string;
}


export interface LoginResponse {
  op: string;
  jwt: string;
}

export enum CommentSortType {
  Hot, Top, New
}

export enum ListingType {
  All, Subscribed, Community
}

export enum ListingSortType {
  Hot, New, TopDay, TopWeek, TopMonth, TopYear, TopAll
}

