-- add follow state denied for private communities
ALTER TYPE community_follower_state
    ADD value 'Denied';

