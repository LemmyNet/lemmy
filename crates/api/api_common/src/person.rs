pub use lemmy_db_schema::{
  newtypes::{LocalUserId, PersonId},
  source::{
    local_user::LocalUser,
    person::{Person, PersonActions},
  },
  PersonContentType,
};
pub use lemmy_db_views_local_user::LocalUserView;
pub use lemmy_db_views_person::{
  api::{GetPersonDetails, GetPersonDetailsResponse, PersonResponse},
  PersonView,
};

pub mod actions {
  pub use lemmy_db_schema::newtypes::PersonContentCombinedId;
  pub use lemmy_db_views_person::api::{BlockPerson, NotePerson};
  pub use lemmy_db_views_person_content_combined::{
    ListPersonContent,
    ListPersonContentResponse,
    PersonContentCombinedView,
  };

  pub mod moderation {
    pub use lemmy_db_schema::{
      newtypes::RegistrationApplicationId,
      source::registration_application::RegistrationApplication,
    };
    pub use lemmy_db_views_person::api::{BanPerson, PurgePerson};
    pub use lemmy_db_views_registration_applications::{
      api::{GetRegistrationApplication, RegistrationApplicationResponse},
      RegistrationApplicationView,
    };
  }
}
