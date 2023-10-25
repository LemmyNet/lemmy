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
  echo "-h Print this help."
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

LOG_PREFIX="[🐀 lemmy]"
ARCH=$(uname -m 2>/dev/null || echo 'unknown') # uname may not exist on windows machines; default to unknown to be safe.

mkdir -p volumes/pictrs

echo "$LOG_PREFIX Please provide your password to change ownership of the pictrs volume."
sudo chown -R 991:991 volumes/pictrs

if [ "$ARCH" = 'arm64' ]; then
  echo "$LOG_PREFIX WARN: If building from images, make sure to uncomment 'platform' in the docker-compose.yml file!"

  # You need a Docker account to pull images. Otherwise, you will get an error like: "error getting credentials"
  if [ -z "$DOCKER_USER" ]; then
      echo "$LOG_PREFIX Logging into Docker Hub..."
      docker login
  else
      echo "$LOG_PREFIX Logging into Docker Hub. Please provide your personal access token."
      docker login --username="$DOCKER_USER"
  fi

  echo "$LOG_PREFIX Initializing images in the background. Please be patient if compiling from source..."
  docker compose up --build
else
  sudo docker compose up --build
fi

echo "$LOG_PREFIX Complete! You can now access the UI at http://localhost:1236."
