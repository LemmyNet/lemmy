use crate::{
  schema::user_,
  user::{UserSafe, User_},
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct UserViewSafe {
  pub user: UserSafe,
  // TODO
  // pub number_of_posts: i64,
  // pub post_score: i64,
  // pub number_of_comments: i64,
  // pub comment_score: i64,
}

pub struct UserViewDangerous {
  pub user: User_,
  // TODO
  // pub number_of_posts: i64,
  // pub post_score: i64,
  // pub number_of_comments: i64,
  // pub comment_score: i64,
}

impl UserViewDangerous {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    let user = user_::table.find(id).first::<User_>(conn)?;
    Ok(Self { user })
  }
}

impl UserViewSafe {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    let user = user_::table.find(id).first::<User_>(conn)?.to_safe();
    Ok(Self { user })
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let admins = user_::table
      // TODO do joins here
      .filter(user_::admin.eq(true))
      .order_by(user_::published)
      .load::<User_>(conn)?;

    Ok(vec_to_user_view_safe(admins))
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let banned = user_::table
      // TODO do joins here
      .filter(user_::banned.eq(true))
      .load::<User_>(conn)?;

    Ok(vec_to_user_view_safe(banned))
  }
}

fn vec_to_user_view_safe(users: Vec<User_>) -> Vec<UserViewSafe> {
  users
    .iter()
    .map(|a| UserViewSafe { user: a.to_safe() })
    .collect::<Vec<UserViewSafe>>()
}
