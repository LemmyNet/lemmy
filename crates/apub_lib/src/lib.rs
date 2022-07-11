// SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
//
// SPDX-License-Identifier: AGPL-3.0-only

pub mod activity_queue;
pub mod data;
pub mod object_id;
pub mod signatures;
pub mod traits;
pub mod utils;
pub mod values;
pub mod verify;

pub static APUB_JSON_CONTENT_TYPE: &str = "application/activity+json";
