use crate::{
  newtypes::{PersonId, LocalUserId};
  schema::community,
  source::{local_user::LocalUser, site::Site},
  CommunityVisibility,
};
use diesel::{
  dsl,
  query_dsl::methods::FilterDsl,
  ExpressionMethods,
}

pub struct Viewer<L, S> {
  person_id: Option<PersonId>,
  local_user: L,
  site_has_content_warning: S,
};

impl From<Option<PersonId>> for Viewer<(), ()> {
  fn from(person_id: Option<PersonId>) -> Self {
    Viewer {
      person_id,
      local_user: (),
      site_has_content_warning: (),
    }
  }
}

impl<'a> From<Option<&'a LocalUser>> for Viewer<Option<&'a LocalUser>, ()> {
  fn from(local_user: Option<&'a LocalUser>) -> Self {
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
    let Viewer { person_id, local_user, .. } = value.into();
    Viewer {
      person_id,
      local_user,
      site_has_content_warning: site.content_warning.is_some(),
    }
  }
}

impl<L, S> Viewer<L, S> {
  /// Hide local only communities from unauthenticated users
  pub fn visible_communities_only<Q>(&self, mut query: Q) -> Q
  where
    Q: FilterDsl<dsl::Eq<community::visibility, CommunityVisibility>, Output = Q>,
  {
    if self.person_id.is_some() {
      query = query.filter(community::visibility.eq(CommunityVisibility::Public));
    }

    query
  }

  pub fn person_id(&self) -> Option<PersonId> {
    self.person_id
  }
}

impl<'a, S> Viewer<Option<&'a LocalUser>, S> {
  pub fn local_user_id(&self) -> Option<LocalUserId> {
    self.local_user.id
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

impl<'a> Viewer<Option<&'a LocalUser>, bool> {
  pub fn show_nsfw(&self) -> bool {
    self.local_user.map(|l| l.show_nsfw).unwrap_or(self.site_has_content_warning)
  }
}
