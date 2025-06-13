use crate::{
  source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  },
  traits::Crud,
  utils::DbPool,
};
use lemmy_utils::error::LemmyResult;

pub struct TestData {
  pub instance: Instance,
  pub site: Site,
}

impl TestData {
  pub async fn create(pool: &mut DbPool<'_>) -> LemmyResult<Self> {
    let instance = Instance::read_or_create(pool, "my_domain.tld".to_string()).await?;

    let site_form = SiteInsertForm::new("test site".to_string(), instance.id);
    let site = Site::create(pool, &site_form).await?;

    let person = Person::create(pool, &PersonInsertForm::test_form(instance.id, "langs")).await?;
    let local_site_form = LocalSiteInsertForm {
      multi_comm_follower: Some(person.id),
      ..LocalSiteInsertForm::new(site.id)
    };
    let local_site = LocalSite::create(pool, &local_site_form).await?;
    LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site.id)).await?;

    Ok(Self { instance, site })
  }

  pub async fn delete(self, pool: &mut DbPool<'_>) -> LemmyResult<()> {
    Instance::delete(pool, self.instance.id).await?;
    Site::delete(pool, self.site.id).await?;
    Ok(())
  }
}
