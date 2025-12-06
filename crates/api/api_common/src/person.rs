pub use lemmy_db_schema::{
  PersonContentType,
  newtypes::LocalUserId,
  source::{
    local_user::LocalUser,
    person::{Person, PersonActions},
  },
};
pub use lemmy_db_schema_file::PersonId;
pub use lemmy_db_views_local_user::LocalUserView;
pub use lemmy_db_views_person::{
  PersonView,
  api::{GetPersonDetails, GetPersonDetailsResponse, PersonResponse},
};

pub mod actions {
  pub use lemmy_db_schema::newtypes::PersonContentCombinedId;
  pub use lemmy_db_views_person::api::{BlockPerson, NotePerson};
  pub use lemmy_db_views_person_content_combined::ListPersonContent;

  pub mod moderation {
    pub use lemmy_db_schema::{
      newtypes::RegistrationApplicationId,
      source::registration_application::RegistrationApplication,
    };
    pub use lemmy_db_views_person::api::{BanPerson, PurgePerson};
    pub use lemmy_db_views_registration_applications::{
      RegistrationApplicationView,
      api::{GetRegistrationApplication, RegistrationApplicationResponse},
    };
  }
}
