use lemmy_db_schema::{
  source::{
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
  },
  traits::Crud,
  utils::DbPool,
};
use lemmy_db_views::structs::LocalUserView;
use lemmy_utils::error::LemmyResult;

#[derive(Default)]
pub struct TestUser {
  pub name: Option<&'static str>,
  pub bio: Option<&'static str>,
  pub admin: bool,
}

impl TestUser {
  pub async fn create(self, pool: &mut DbPool<'_>) -> LemmyResult<LocalUserView> {
    let instance_id = Instance::read_or_create(pool, "example.com".to_string())
      .await?
      .id;
    let name = self
      .name
      .map_or_else(|| uuid::Uuid::new_v4().to_string(), ToString::to_string);

    let person_form = PersonInsertForm {
      display_name: Some(name.clone()),
      bio: self.bio.map(ToString::to_string),
      ..PersonInsertForm::test_form(instance_id, &name)
    };
    let person = Person::create(pool, &person_form).await?;

    let user_form = match self.admin {
      true => LocalUserInsertForm::test_form_admin(person.id),
      false => LocalUserInsertForm::test_form(person.id),
    };
    let local_user = LocalUser::create(pool, &user_form, vec![]).await?;

    Ok(LocalUserView::read(pool, local_user.id).await?)
  }
}
