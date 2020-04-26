use super::*;

fn populate_object_props(
  props: &mut ObjectProperties,
  addressed_to: &str,
  object_id: &str,
) -> Result<(), Error> {
  props
    .set_context_xsd_any_uri(context())?
    // TODO: the activity needs a seperate id from the object
    .set_id(object_id)?
    // TODO: should to/cc go on the Create, or on the Post? or on both?
    // TODO: handle privacy on the receiving side (at least ignore anything thats not public)
    .set_to_xsd_any_uri(public())?
    .set_cc_xsd_any_uri(addressed_to)?;
  Ok(())
}

/// Send an activity to a list of recipients, using the correct headers etc.
pub fn send_activity<A>(
  activity: &A,
  private_key: &str,
  sender_id: &str,
  to: Vec<String>,
) -> Result<(), Error>
where
  A: Serialize + Debug,
{
  let json = serde_json::to_string(&activity)?;
  debug!("Sending activitypub activity {} to {:?}", json, to);
  // TODO it needs to expand, the to field needs to expand and dedup the followers urls
  // The inbox is determined by first retrieving the target actor's JSON-LD representation and then looking up the inbox property. If a recipient is a Collection or OrderedCollection, then the server MUST dereference the collection (with the user's credentials) and discover inboxes for each item in the collection. Servers MUST limit the number of layers of indirections through collections which will be performed, which MAY be one.
  for t in to {
    let to_url = Url::parse(&t)?;
    if !is_apub_id_valid(&to_url) {
      debug!("Not sending activity to {} (invalid or blacklisted)", t);
      continue;
    }
    let request = Request::post(t).header("Host", to_url.domain().unwrap());
    let signature = sign(&request, private_key, sender_id)?;
    let res = request
      .header("Signature", signature)
      .header("Content-Type", "application/json")
      .body(json.to_owned())?
      .send()?;
    debug!("Result for activity send: {:?}", res);
  }
  Ok(())
}

/// For a given community, returns the inboxes of all followers.
fn get_follower_inboxes(conn: &PgConnection, community: &Community) -> Result<Vec<String>, Error> {
  Ok(
    CommunityFollowerView::for_community(conn, community.id)?
      .into_iter()
      .filter(|c| !c.user_local)
      // TODO eventually this will have to use the inbox or shared_inbox column, meaning that view
      // will have to change
      .map(|c| format!("{}/inbox", c.user_actor_id.to_owned()))
      .unique()
      .collect(),
  )
}

/// Send out information about a newly created post, to the followers of the community.
pub fn send_post_create(post: &Post, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
  let page = post.to_apub(conn)?;
  let community = Community::read(conn, post.community_id)?;
  let mut create = Create::new();
  populate_object_props(
    &mut create.object_props,
    &community.get_followers_url(),
    &post.ap_id,
  )?;
  create
    .create_props
    .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
    .set_object_base_box(page)?;
  send_activity(
    &create,
    &creator.private_key.as_ref().unwrap(),
    &creator.actor_id,
    get_follower_inboxes(conn, &community)?,
  )?;
  Ok(())
}

/// Send out information about an edited post, to the followers of the community.
pub fn send_post_update(post: &Post, creator: &User_, conn: &PgConnection) -> Result<(), Error> {
  let page = post.to_apub(conn)?;
  let community = Community::read(conn, post.community_id)?;
  let mut update = Update::new();
  populate_object_props(
    &mut update.object_props,
    &community.get_followers_url(),
    &post.ap_id,
  )?;
  update
    .update_props
    .set_actor_xsd_any_uri(creator.actor_id.to_owned())?
    .set_object_base_box(page)?;
  send_activity(
    &update,
    &creator.private_key.as_ref().unwrap(),
    &creator.actor_id,
    get_follower_inboxes(conn, &community)?,
  )?;
  Ok(())
}
