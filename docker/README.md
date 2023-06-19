# Lemmy With Docker

This directory contains the files and assets needed to run Lemmy via [Docker](https://www.docker.com/).

## Requirements

- A working [Docker installation](https://docs.docker.com/engine/install/) with the compose plugin available (installed by default)
- About 4GB of available memory to build, and about half a GB to run
- Adjust the variables declared in [lemmy.hjson](lemmy.hjson) as required

## For Testing and Development

- Run the following command from this directory

  ```sh
  docker compose up
  ```

## For Production

- Run the following command from this directory

  ```sh
  docker compose -f docker-compose.yml -f docker-compose.prod.yml up
  ```
