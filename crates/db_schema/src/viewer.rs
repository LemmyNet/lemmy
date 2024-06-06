use crate::{
  newtypes::{LocalUserId, PersonId},
  schema::community,
  source::{local_user::LocalUser, site::Site},
  CommunityVisibility,
};
use diesel::{dsl, query_dsl::methods::FilterDsl, ExpressionMethods};

/// Hide local only communities from unauthenticated users
///
/// TODO: change `read` functions to take `impl Viewer` instead of `Option<PersonId>`,
/// and move this function to `Viewer`
fn visible_communities_only<T, Q>(local_user: Option<T>, query: Q) -> Q
where
  Q: FilterDsl<dsl::Eq<community::visibility, CommunityVisibility>, Output = Q>,
{
  if local_user.is_none() {
    query.filter(community::visibility.eq(CommunityVisibility::Public))
  } else {
    query
  }
}

trait Viewer {
  fn local_user(&self) -> Option<&LocalUser>;

  fn person_id(&self) -> Option<PersonId> {
    self.local_user().map(|l| l.person_id)
  }

  fn local_user_id(&self) -> Option<LocalUserId> {
    self.local_user().map(|l| l.id)
  }

  fn show_bot_accounts(&self) -> bool {
    self
      .local_user()
      .map(|l| l.show_bot_accounts)
      .unwrap_or(true)
  }

  fn show_read_posts(&self) -> bool {
    self.local_user().map(|l| l.show_read_posts).unwrap_or(true)
  }

  fn is_admin(&self) -> bool {
    self.local_user().map(|l| l.admin).unwrap_or(false)
  }

  fn show_nsfw(&self, site: &Site) -> bool {
    self
      .local_user()
      .map(|l| l.show_nsfw)
      .unwrap_or(site.content_warning.is_some())
  }
}

impl<'a, T: Into<&'a LocalUser>> Viewer for Option<T> {
  fn local_user(&self) -> Option<&LocalUser> {
    self.map(Into::into)
  }
}
