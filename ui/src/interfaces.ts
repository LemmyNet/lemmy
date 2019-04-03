export enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, ListCategories, GetPost, GetCommunity, CreateComment, EditComment, CreateCommentLike, GetPosts, CreatePostLike, EditPost, EditCommunity
}

export interface User {
  id: number;
  iss: string;
  username: string;
}

export interface Community {
  id: number;
  name: string;
  title: string;
  description?: string;
  creator_id: number;
  creator_name: string;
  category_id: number;
  category_name: string;
  number_of_subscribers: number;
  number_of_posts: number;
  published: string;
  updated?: string;
}

export interface CommunityForm {
  name: string;
  title: string;
  description?: string,
  category_id: number,
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

export interface ListCategoriesResponse {
  op: string;
  categories: Array<Category>;
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
  edit_id?: number;
  auth: string;
}

export interface GetPostResponse {
  op: string;
  post: Post;
  comments: Array<Comment>;
  community: Community;
}

export interface PostResponse {
  op: string;
  post: Post;
}

export interface Comment {
  id: number;
  content: string;
  creator_id: number;
  creator_name: string;
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

export interface Category {
  id: number;
  name: string;
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

