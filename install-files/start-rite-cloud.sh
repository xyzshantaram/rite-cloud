#!/usr/bin/env bash

# EDIT THESE VARIABLES BEFORE USING THIS SCRIPT!
export CLIENT_ID=
export CLIENT_SECRET=
export APP_URL=
export TOKEN_URL=
export AUTH_URL=
export REDIRECT_URL=
export TIDE_SECRET=
export FILE_LIMIT=
export RITE_DB_URL="sqlite:///opt/rite-cloud/storage/rite.db?mode=rwc"
export SESSION_DB_URL="sqlite:///opt/rite-cloud/storage/sessions.db?mode=rwc"

echo "Starting rite-cloud..."
cd /opt/rite-cloud/
/opt/rite-cloud/rite-cloud