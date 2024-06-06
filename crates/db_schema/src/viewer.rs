use crate::{
  newtypes::{LocalUserId, PersonId},
  schema::community,
  source::{local_user::LocalUser, site::Site},
  CommunityVisibility,
};
use diesel::{dsl, query_dsl::methods::FilterDsl, ExpressionMethods};

/// Wraps `Option<PersonId>` or `Option<&LocalUser>`
pub struct Viewer<T>(Option<T>);

impl From<Option<PersonId>> for Viewer<PersonId> {
  fn from(person_id: Option<PersonId>) -> Self {
    Viewer(person_id)
  }
}

impl<'a, T> From<Option<&'a T>> for Viewer<&'a LocalUser>
where
  &'a T: Into<&'a LocalUser>,
{
  fn from(local_user: Option<&'a T>) -> Self {
    Viewer(local_user.map(Into::into))
  }
}

impl<T> Viewer<T> {
  /// Hide local only communities from unauthenticated users
  pub fn visible_communities_only<Q>(&self, query: Q) -> Q
  where
    Q: FilterDsl<dsl::Eq<community::visibility, CommunityVisibility>, Output = Q>,
  {
    if self.0.is_none() {
      query.filter(community::visibility.eq(CommunityVisibility::Public))
    } else {
      query
    }
  }
}

impl<'a> Viewer<&'a LocalUser> {
  pub fn local_user_id(&self) -> Option<LocalUserId> {
    self.0.map(|l| l.id)
  }

  pub fn show_bot_accounts(&self) -> bool {
    self.0.map(|l| l.show_bot_accounts).unwrap_or(true)
  }

  pub fn show_read_posts(&self) -> bool {
    self.0.map(|l| l.show_read_posts).unwrap_or(true)
  }

  pub fn is_admin(&self) -> bool {
    self.0.map(|l| l.admin).unwrap_or(false)
  }

  pub fn show_nsfw(&self, site: &Site) -> bool {
    self
      .0
      .map(|l| l.show_nsfw)
      .unwrap_or(site.content_warning.is_some())
  }
}
