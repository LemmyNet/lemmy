use crate::Perform;
use actix_web::web::Data;
use lemmy_api_common::{
  context::LemmyContext,
  person::{LoginResponse, SaveUserSettings},
  utils::{local_user_view_from_jwt, send_verification_email},
};
use lemmy_db_schema::{
  source::{
    actor_language::LocalUserLanguage,
    local_user::{LocalUser, LocalUserUpdateForm},
    person::{Person, PersonUpdateForm},
    post::PostRead,
  },
  traits::{Crud, Readable},
  utils::{diesel_option_overwrite, diesel_option_overwrite_to_url, DbPool},
};
use lemmy_db_views::structs::{LocalUserView, SiteView};
use lemmy_utils::{
  claims::Claims,
  error::LemmyError,
  utils::validation::{
    build_totp_2fa,
    generate_totp_2fa_secret,
    is_valid_bio_field,
    is_valid_display_name,
    is_valid_matrix_id,
  },
};

#[async_trait::async_trait(?Send)]
impl Perform for SaveUserSettings {
  type Response = LoginResponse;

  #[tracing::instrument(skip(context))]
  async fn perform(&self, context: &Data<LemmyContext>) -> Result<LoginResponse, LemmyError> {
    let data: &SaveUserSettings = self;
    let local_user_view = local_user_view_from_jwt(&data.auth, context).await?;
    let site_view = SiteView::read_local(context.pool()).await?;

    let avatar = diesel_option_overwrite_to_url(&data.avatar)?;
    let banner = diesel_option_overwrite_to_url(&data.banner)?;
    let bio = diesel_option_overwrite(&data.bio);
    let display_name = diesel_option_overwrite(&data.display_name);
    let matrix_user_id = diesel_option_overwrite(&data.matrix_user_id);
    let email_deref = data.email.as_deref().map(str::to_lowercase);
    let email = diesel_option_overwrite(&email_deref);

    if let Some(Some(email)) = &email {
      let previous_email = local_user_view.local_user.email.clone().unwrap_or_default();
      // Only send the verification email if there was an email change
      if previous_email.ne(email) {
        send_verification_email(&local_user_view, email, context.pool(), context.settings())
          .await?;
      }
    }

    // When the site requires email, make sure email is not Some(None). IE, an overwrite to a None value
    if let Some(email) = &email {
      if email.is_none() && site_view.local_site.require_email_verification {
        return Err(LemmyError::from_message("email_required"));
      }
    }

    if let Some(Some(bio)) = &bio {
      is_valid_bio_field(bio)?;
    }

    if let Some(Some(display_name)) = &display_name {
      is_valid_display_name(
        display_name.trim(),
        site_view.local_site.actor_name_max_length as usize,
      )?;
    }

    if let Some(Some(matrix_user_id)) = &matrix_user_id {
      is_valid_matrix_id(matrix_user_id)?;
    }

    let local_user_id = local_user_view.local_user.id;
    let person_id = local_user_view.person.id;
    let default_listing_type = data.default_listing_type;
    let default_sort_type = data.default_sort_type;

    let person_form = PersonUpdateForm::builder()
      .display_name(display_name)
      .bio(bio)
      .matrix_user_id(matrix_user_id)
      .bot_account(data.bot_account)
      .avatar(avatar)
      .banner(banner)
      .build();

    Person::update(context.pool(), person_id, &person_form)
      .await
      .map_err(|e| LemmyError::from_error_message(e, "user_already_exists"))?;

    if let Some(discussion_languages) = data.discussion_languages.clone() {
      LocalUserLanguage::update(context.pool(), discussion_languages, local_user_id).await?;
    }

    // If generate_totp is Some(false), this will clear it out from the database.
    let (totp_2fa_secret, totp_2fa_url) = if let Some(generate) = data.generate_totp_2fa {
      if generate {
        let secret = generate_totp_2fa_secret();
        let url =
          build_totp_2fa(&site_view.site.name, &local_user_view.person.name, &secret)?.get_url();
        (Some(Some(secret)), Some(Some(url)))
      } else {
        (Some(None), Some(None))
      }
    } else {
      (None, None)
    };

    handle_save_read_posts_change(self, &local_user_view, context.pool()).await?;

    let local_user_form = LocalUserUpdateForm::builder()
      .email(email)
      .show_avatars(data.show_avatars)
      .save_read_posts(data.save_read_posts)
      .show_read_posts(data.show_read_posts)
      .show_new_post_notifs(data.show_new_post_notifs)
      .send_notifications_to_email(data.send_notifications_to_email)
      .show_nsfw(data.show_nsfw)
      .show_bot_accounts(data.show_bot_accounts)
      .show_scores(data.show_scores)
      .default_sort_type(default_sort_type)
      .default_listing_type(default_listing_type)
      .theme(data.theme.clone())
      .interface_language(data.interface_language.clone())
      .totp_2fa_secret(totp_2fa_secret)
      .totp_2fa_url(totp_2fa_url)
      .open_links_in_new_tab(data.open_links_in_new_tab)
      .build();

    let local_user_res = LocalUser::update(context.pool(), local_user_id, &local_user_form).await;
    let updated_local_user = match local_user_res {
      Ok(u) => u,
      Err(e) => {
        let err_type = if e.to_string()
          == "duplicate key value violates unique constraint \"local_user_email_key\""
        {
          "email_already_exists"
        } else {
          "user_already_exists"
        };

        return Err(LemmyError::from_error_message(e, err_type));
      }
    };

    // Return the jwt
    Ok(LoginResponse {
      jwt: Some(
        Claims::jwt(
          updated_local_user.id.0,
          &context.secret().jwt_secret,
          &context.settings().hostname,
        )?
        .into(),
      ),
      verify_email_sent: false,
      registration_created: false,
    })
  }
}

async fn handle_save_read_posts_change(
  data: &SaveUserSettings,
  local_user_view: &LocalUserView,
  pool: &DbPool,
) -> Result<usize, LemmyError> {
  if let Some(save_read_posts) = data.save_read_posts {
    if !save_read_posts && local_user_view.local_user.save_read_posts {
      // Delete all read posts
      return PostRead::delete_all(pool, &local_user_view.person.id)
        .await
        .map_err(|e| LemmyError::from_error_message(e, "couldnt_mark_delete_past_read_posts"));
    }
  }

  Ok(0)
}

#[cfg(test)]
mod tests {
  use crate::local_user::save_settings::handle_save_read_posts_change;
  use lemmy_api_common::person::SaveUserSettings;
  use lemmy_db_schema::{
    aggregates::structs::PersonAggregates,
    source::{
      community::{Community, CommunityInsertForm},
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      post::{Post, PostInsertForm, PostRead, PostReadForm},
    },
    traits::{Crud, Readable},
    utils::build_db_pool_for_tests,
  };
  use lemmy_db_views::structs::LocalUserView;

  #[tokio::test]
  async fn test_disable_save_read_posts() {
    let pool = &build_db_pool_for_tests().await;

    let person_name = "tegan".to_string();

    let inserted_instance = Instance::read_or_create(pool, "my_domain.tld".to_string())
      .await
      .unwrap();

    let new_community = CommunityInsertForm::builder()
      .name("test_community_3".to_string())
      .title("nada".to_owned())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_community = Community::create(pool, &new_community).await.unwrap();

    let new_person = PersonInsertForm::builder()
      .name(person_name.clone())
      .public_key("pubkey".to_string())
      .instance_id(inserted_instance.id)
      .build();

    let inserted_person = Person::create(pool, &new_person).await.unwrap();

    let local_user_form = LocalUserInsertForm::builder()
      .person_id(inserted_person.id)
      .password_encrypted(String::new())
      .save_read_posts(Some(true))
      .build();
    let inserted_local_user = LocalUser::create(pool, &local_user_form).await.unwrap();

    let local_user_view = LocalUserView {
      local_user: inserted_local_user.clone(),
      person: inserted_person.clone(),
      counts: PersonAggregates::default(),
    };

    // Insert a read post
    let new_post = PostInsertForm::builder()
      .name("A test post".into())
      .creator_id(inserted_person.id)
      .community_id(inserted_community.id)
      .build();

    let inserted_post = Post::create(pool, &new_post).await.unwrap();

    let post_read_form = PostReadForm {
      post_id: inserted_post.id,
      person_id: inserted_person.id,
    };

    PostRead::mark_as_read(pool, &post_read_form).await.unwrap();

    let save_user_settings = SaveUserSettings {
      save_read_posts: Some(false),
      ..Default::default()
    };

    let read_removed = handle_save_read_posts_change(&save_user_settings, &local_user_view, pool)
      .await
      .unwrap();

    assert_eq!(read_removed, 1);

    let redundant_read_removed = PostRead::mark_as_unread(pool, &post_read_form)
      .await
      .unwrap();

    // Should be 0 since posts are already removed
    assert_eq!(redundant_read_removed, 0);
  }
}
