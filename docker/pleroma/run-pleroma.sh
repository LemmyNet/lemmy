#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2019-2022 2019 Felix Ableitner, <me@nutomic.com> et al.
#
# SPDX-License-Identifier: AGPL-3.0-only

set -euo pipefail

if [ ! -e "$PLEROMA_CONFIG_PATH" ] ; then
  generate-pleroma-config.sh
fi

while ! pg_isready -U "${POSTGRES_USER:-pleroma}" -d "postgres://${POSTGRES_HOST:-postgres}:5432/${POSTGRES_DB:-pleroma}" -t 1; do
  echo "Waiting for ${POSTGRES_HOST-postgres} to come up..." >&2
  sleep 1s
done

pleroma_ctl migrate

if [ "${USE_RUM:-n}" = "y" ] ; then
  pleroma_ctl migrate --migrations-path priv/repo/optional_migrations/rum_indexing/
fi

if [ "${USE_SOAPBOX:-n}" = "y" ]; then
  unzip -o /tmp/soapbox-fe.zip -d /var/lib/pleroma
  rm /tmp/soapbox-fe.zip
fi

exec pleroma start
