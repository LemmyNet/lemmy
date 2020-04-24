use super::*;

#[derive(Deserialize)]
pub struct CommunityQuery {
  community_name: String,
}

impl ToApub<GroupExt> for Community {
  // Turn a Lemmy Community into an ActivityPub group that can be sent out over the network.
  fn to_apub(&self, conn: &PgConnection) -> Result<GroupExt, Error> {
    let mut group = Group::default();
    let oprops: &mut ObjectProperties = group.as_mut();

    let creator = User_::read(conn, self.creator_id)?;
    oprops
      .set_context_xsd_any_uri(context())?
      .set_id(self.actor_id.to_owned())?
      .set_name_xsd_string(self.name.to_owned())?
      .set_published(convert_datetime(self.published))?
      .set_attributed_to_xsd_any_uri(creator.actor_id)?;

    if let Some(u) = self.updated.to_owned() {
      oprops.set_updated(convert_datetime(u))?;
    }
    if let Some(d) = self.description.to_owned() {
      // TODO: this should be html, also add source field with raw markdown
      //       -> same for post.content and others
      oprops.set_summary_xsd_string(d)?;
    }

    let mut actor_props = ApActorProperties::default();

    actor_props
      .set_preferred_username(self.title.to_owned())?
      .set_inbox(self.get_inbox_url())?
      .set_outbox(self.get_outbox_url())?
      .set_followers(self.get_followers_url())?;

    let public_key = PublicKey {
      id: format!("{}#main-key", self.actor_id),
      owner: self.actor_id.to_owned(),
      public_key_pem: self.public_key.to_owned().unwrap(),
    };

    Ok(group.extend(actor_props).extend(public_key.to_ext()))
  }
}

impl ActorType for Community {
  fn actor_id(&self) -> String {
    self.actor_id.to_owned()
  }
}

impl FromApub<GroupExt> for CommunityForm {
  /// Parse an ActivityPub group received from another instance into a Lemmy community.
  fn from_apub(group: &GroupExt, conn: &PgConnection) -> Result<Self, Error> {
    let oprops = &group.base.base.object_props;
    let aprops = &group.base.extension;
    let public_key: &PublicKey = &group.extension.public_key;

    let _followers_uri = Url::parse(&aprops.get_followers().unwrap().to_string())?;
    let _outbox_uri = Url::parse(&aprops.get_outbox().to_string())?;
    // TODO don't do extra fetching here
    // let _outbox = fetch_remote_object::<OrderedCollection>(&outbox_uri)?;
    // let _followers = fetch_remote_object::<UnorderedCollection>(&followers_uri)?;
    let apub_id = &oprops.get_attributed_to_xsd_any_uri().unwrap().to_string();
    let creator = get_or_fetch_and_upsert_remote_user(&apub_id, conn)?;

    Ok(CommunityForm {
      name: oprops.get_name_xsd_string().unwrap().to_string(),
      title: aprops.get_preferred_username().unwrap().to_string(),
      // TODO: should be parsed as html and tags like <script> removed (or use markdown source)
      //       -> same for post.content etc
      description: oprops.get_content_xsd_string().map(|s| s.to_string()),
      category_id: 1, // -> peertube uses `"category": {"identifier": "9","name": "Comedy"},`
      creator_id: creator.id,
      removed: None,
      published: oprops
        .get_published()
        .map(|u| u.as_ref().to_owned().naive_local()),
      updated: oprops
        .get_updated()
        .map(|u| u.as_ref().to_owned().naive_local()),
      deleted: None,
      nsfw: false,
      actor_id: oprops.get_id().unwrap().to_string(),
      local: false,
      private_key: None,
      public_key: Some(public_key.to_owned().public_key_pem),
      last_refreshed_at: Some(naive_now()),
    })
  }
}

/// Return the community json over HTTP.
pub async fn get_apub_community_http(
  info: Path<CommunityQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, &info.community_name)?;
  let c = community.to_apub(&db.get().unwrap())?;
  Ok(create_apub_response(&c))
}

/// Returns an empty followers collection, only populating the siz (for privacy).
// TODO this needs to return the actual followers, and the to: field needs this
pub async fn get_apub_community_followers(
  info: Path<CommunityQuery>,
  db: DbPoolParam,
) -> Result<HttpResponse<Body>, Error> {
  let community = Community::read_from_name(&&db.get()?, &info.community_name)?;

  let conn = db.get()?;

  //As we are an object, we validated that the community id was valid
  let community_followers = CommunityFollowerView::for_community(&conn, community.id).unwrap();

  let mut collection = UnorderedCollection::default();
  let oprops: &mut ObjectProperties = collection.as_mut();
  oprops
    .set_context_xsd_any_uri(context())?
    .set_id(community.actor_id)?;
  collection
    .collection_props
    .set_total_items(community_followers.len() as u64)?;
  Ok(create_apub_response(&collection))
}

// TODO should not be doing this
// Returns an UnorderedCollection with the latest posts from the community.
//pub async fn get_apub_community_outbox(
//  info: Path<CommunityQuery>,
//  db: DbPoolParam,
//  chat_server: ChatServerParam,
//) -> Result<HttpResponse<Body>, Error> {
//  let community = Community::read_from_name(&&db.get()?, &info.community_name)?;

//  let conn = establish_unpooled_connection();
//  //As we are an object, we validated that the community id was valid
//  let community_posts: Vec<Post> = Post::list_for_community(&conn, community.id)?;

//  let mut collection = OrderedCollection::default();
//  let oprops: &mut ObjectProperties = collection.as_mut();
//  oprops
//    .set_context_xsd_any_uri(context())?
//    .set_id(community.actor_id)?;
//  collection
//    .collection_props
//    .set_many_items_base_boxes(
//      community_posts
//        .iter()
//        .map(|c| c.as_page(&conn).unwrap())
//        .collect(),
//    )?
//    .set_total_items(community_posts.len() as u64)?;

//  Ok(create_apub_response(&collection))
//}
