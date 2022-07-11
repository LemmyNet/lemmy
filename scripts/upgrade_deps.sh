#!/bin/bash

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

pushd ../

# Check unused deps
cargo udeps --all-targets

# Upgrade deps
cargo upgrade --workspace

# Run check
cargo check

popd
