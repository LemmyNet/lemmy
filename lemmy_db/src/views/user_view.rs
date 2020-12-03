use crate::{
  aggregates::user_aggregates::UserAggregates,
  schema::{user_, user_aggregates},
  user::{UserSafe, User_},
};
use diesel::{result::Error, *};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct UserViewSafe {
  pub user: UserSafe,
  pub counts: UserAggregates,
}

#[derive(Debug, Serialize, Clone)]
pub struct UserViewDangerous {
  pub user: User_,
  pub counts: UserAggregates,
}

impl UserViewDangerous {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    let (user, counts) = user_::table
      .find(id)
      .inner_join(user_aggregates::table)
      .first::<(User_, UserAggregates)>(conn)?;
    Ok(Self { user, counts })
  }
}

impl UserViewSafe {
  pub fn read(conn: &PgConnection, id: i32) -> Result<Self, Error> {
    let (user, counts) = user_::table
      .find(id)
      .inner_join(user_aggregates::table)
      .first::<(User_, UserAggregates)>(conn)?;
    Ok(Self {
      user: user.to_safe(),
      counts,
    })
  }

  pub fn admins(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let admins = user_::table
      .inner_join(user_aggregates::table)
      .filter(user_::admin.eq(true))
      .order_by(user_::published)
      .load::<(User_, UserAggregates)>(conn)?;

    Ok(vec_to_user_view_safe(admins))
  }

  pub fn banned(conn: &PgConnection) -> Result<Vec<Self>, Error> {
    let banned = user_::table
      .inner_join(user_aggregates::table)
      .filter(user_::banned.eq(true))
      .load::<(User_, UserAggregates)>(conn)?;

    Ok(vec_to_user_view_safe(banned))
  }
}

fn vec_to_user_view_safe(users: Vec<(User_, UserAggregates)>) -> Vec<UserViewSafe> {
  users
    .iter()
    .map(|a| UserViewSafe {
      user: a.0.to_safe(),
      counts: a.1.to_owned(),
    })
    .collect::<Vec<UserViewSafe>>()
}
