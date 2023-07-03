#!/bin/sh
set -e

Help()
{
  # Display help
  echo "Usage: ./docker_update.sh [OPTIONS]"
  echo ""
  echo "Start all docker containers required to run Lemmy."
  echo ""
  echo "Options:"
  echo "-u Docker username. Only required if managing Docker via Docker Desktop with a personal access token."
  echo "-h Print this help"
}

while getopts ":hu:" option; do
  case $option in
    h) Help
       exit;;
    u) DOCKER_USER=$OPTARG
       ;;
    *) echo "Invalid option $OPTARG."
       exit;;
  esac
done

# uname may not exist on windows machines; default to unknown to be safe.
ARCH=$(uname -m 2>/dev/null || echo 'unknown')

mkdir -p volumes/pictrs

echo "[🐀 lemmy] Need user password to run 'sudo chown' for the pictrs volume."
sudo chown -R 991:991 volumes/pictrs

if [ "$ARCH" = 'arm64' ]; then
  echo "[🐀 lemmy] WARN: If building from images, make sure to uncomment 'platform' in the docker-compose.yml file!"

  # You need a Docker account to pull images. Otherwise, you will get an error like: "error getting credentials"
  if [ -z "$DOCKER_USER" ]; then
      echo "[🐀 lemmy] Logging into Docker Hub..."
      docker login
  else
      echo "[🐀 lemmy] Logging into Docker Hub. Please provide your personal access token."
      docker login --username="$DOCKER_USER"
  fi

  echo "[🐀 lemmy] Initializing images in the background. Please be patient if compiling from source..."
  docker compose up -d --build
else
  sudo docker compose up -d --build
fi

echo "[🐀 lemmy] Complete! You can now access the UI at http://localhost:1236."
