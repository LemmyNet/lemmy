#!/bin/bash
set -e

if [[ $(id -u) != 0 ]]; then
    echo "This migration needs to be run as root"
    exit
fi

if [[ ! -f docker-compose.yml ]]; then
    echo "No docker-compose.yml found in current directory. Is this the right folder?"
    exit
fi

if ! which jq > /dev/null; then
    echo "jq must be installed to run this migration. On ubuntu systems, try 'sudo apt-get install jq'"
    exit
fi

# Fixing pictrs permissions
mkdir -p volumes/pictrs
sudo chown -R 991:991 volumes/pictrs

echo "Restarting docker-compose, making sure that pictrs is started and pictshare is removed"
docker-compose up -d --remove-orphans

if [[ -z $(docker-compose ps | grep pictrs) ]]; then
    echo "Pict-rs is not running, make sure you update Lemmy first"
    exit
fi

# echo "Stopping Lemmy so that users dont upload new images during the migration"
# docker-compose stop lemmy

CRASHED_ON=()

pushd volumes/pictshare/
echo "Importing pictshare images to pict-rs..."
IMAGE_NAMES=*
for image in $IMAGE_NAMES; do
    IMAGE_PATH="$(pwd)/$image/$image"
    if [[ ! -f $IMAGE_PATH ]]; then
        continue
    fi
    res=$(curl -s -F "images[]=@$IMAGE_PATH" http://127.0.0.1:8537/import | jq .msg)
    if [ "${res}" == "" ]; then
        echo -n "C" >&2
        echo ""
        CRASHED_ON+=("${IMAGE_PATH}")
        echo "Failed to import $IMAGE_PATH with no error message"
        echo "  assuming crash, sleeping"
        sleep 10
        continue
    fi
    if [ "${res}" != "\"ok\"" ]; then
        echo -n "F" >&2
        echo ""
        echo "Failed to import $IMAGE_PATH"
        echo "  Reason: ${res}"
    else
        echo -n "." >&2
    fi
done

for image in ${CRASHED_ON[@]}; do
    echo "Retrying ${image}"
    res=$(curl -s -F "images[]=@$IMAGE_PATH" http://127.0.0.1:8537/import | jq .msg)
    if [ "${res}" != "\"ok\"" ]; then
        echo -n "F" >&2
        echo ""
        echo "Failed to upload ${image} on 2nd attempt"
        echo "  Reason: ${res}"
    else
        echo -n "." >&2
    fi
done

echo "Fixing permissions on pictshare folder"
find . -type d -exec chmod 755 {} \;
find . -type f -exec chmod 644 {} \;

popd

echo "Rewrite image links in Lemmy database"
docker-compose exec -u  postgres postgres psql -U lemmy -c "UPDATE user_ SET avatar = REPLACE(avatar, 'pictshare', 'pictrs/image') WHERE avatar is not null;"
docker-compose exec -u  postgres postgres psql -U lemmy -c "UPDATE post SET url = REPLACE(url, 'pictshare', 'pictrs/image') WHERE url is not null;"

echo "Moving pictshare data folder to pictshare_backup"
mv volumes/pictshare volumes/pictshare_backup

echo "Migration done, starting Lemmy again"
echo "If everything went well, you can delete ./volumes/pictshare_backup/"
docker-compose start lemmy
