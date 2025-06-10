use crate::ReadableFederationState;
use lemmy_db_schema::source::federation_queue_state::FederationQueueState;
use lemmy_utils::federate_retry_sleep_duration;

#[allow(clippy::expect_used)]
impl From<FederationQueueState> for ReadableFederationState {
  fn from(internal_state: FederationQueueState) -> Self {
    ReadableFederationState {
      next_retry: internal_state.last_retry.map(|r| {
        r + chrono::Duration::from_std(federate_retry_sleep_duration(internal_state.fail_count))
          .expect("sleep duration longer than 2**63 ms (262 million years)")
      }),
      internal_state,
    }
  }
}
