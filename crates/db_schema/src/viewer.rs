use crate::{
  newtypes::{LocalUserId, PersonId},
  schema::community,
  source::{local_user::LocalUser, site::Site},
  CommunityVisibility,
};
use diesel::{dsl, query_dsl::methods::FilterDsl, ExpressionMethods};

pub struct Viewer<L, S> {
  person_id: Option<PersonId>,
  /// `Option<&LocalUser>` or `()`, depending on what type was converted to `Viewer`
  local_user: L,
  /// `bool` or `()`, depending on what type was converted to `Viewer`
  site_has_content_warning: S,
}

impl From<Option<PersonId>> for Viewer<(), ()> {
  fn from(person_id: Option<PersonId>) -> Self {
    Viewer {
      person_id,
      local_user: (),
      site_has_content_warning: (),
    }
  }
}

impl<'a, T> From<Option<&'a T>> for Viewer<Option<&'a LocalUser>, ()>
where
  &'a T: Into<&'a LocalUser>,
{
  fn from(local_user: Option<&'a T>) -> Self {
    let local_user = local_user.map(Into::into);
    Viewer {
      person_id: local_user.map(|l| l.person_id),
      local_user,
      site_has_content_warning: (),
    }
  }
}

impl<'a, T, L> From<(T, &'a Site)> for Viewer<L, bool>
where
  Viewer<L, ()>: From<T>,
{
  fn from((value, site): (T, &'a Site)) -> Self {
    let viewer = Viewer::from(value);
    Viewer {
      person_id: viewer.person_id,
      local_user: viewer.local_user,
      site_has_content_warning: site.content_warning.is_some(),
    }
  }
}

// Methods that are always available
impl<L, S> Viewer<L, S> {
  /// Hide local only communities from unauthenticated users
  pub fn visible_communities_only<Q>(&self, query: Q) -> Q
  where
    Q: FilterDsl<dsl::Eq<community::visibility, CommunityVisibility>, Output = Q>,
  {
    if self.person_id.is_none() {
      query.filter(community::visibility.eq(CommunityVisibility::Public));
    } else {
        query
    }
  }

  pub fn person_id(&self) -> Option<PersonId> {
    self.person_id
  }
}

// Methods that can only work as expected if `local_user` is set
impl<'a, S> Viewer<Option<&'a LocalUser>, S> {
  pub fn local_user_id(&self) -> Option<LocalUserId> {
    self.local_user.map(|l| l.id)
  }

  pub fn show_bot_accounts(&self) -> bool {
    self.local_user.map(|l| l.show_bot_accounts).unwrap_or(true)
  }

  pub fn show_read_posts(&self) -> bool {
    self.local_user.map(|l| l.show_read_posts).unwrap_or(true)
  }

  pub fn is_admin(&self) -> bool {
    self.local_user.map(|l| l.admin).unwrap_or(false)
  }
}

// Methods that can only work as expected if `local_user` and `site_has_content_warning` are set
impl<'a> Viewer<Option<&'a LocalUser>, bool> {
  pub fn show_nsfw(&self) -> bool {
    self
      .local_user
      .map(|l| l.show_nsfw)
      .unwrap_or(self.site_has_content_warning)
  }
}
