export enum UserOperation {
  Login, Register, CreateCommunity, CreatePost, ListCommunities, GetPost, GetCommunity, CreateComment, CreateCommentLike
}

export interface User {
  id: number;
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
  id: number;
  name: string;
  url?: string;
  body?: string;
  attributed_to: string;
  community_id: number;
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
  attributed_to: string;
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


