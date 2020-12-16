# Creating a Custom Frontend

The backend and frontend are completely decoupled, and run in independent Docker containers. They only communicate over the [Lemmy API](websocket_http_api.md), which makes it quite easy to write alternative frontends.

This creates a lot of potential for custom frontends, which could change much of the design and user experience of Lemmy. For example, it would be possible to create a frontend in the style of a traditional forum like [phpBB](https://www.phpbb.com/), or a question-and-answer site like [stackoverflow](https://stackoverflow.com/). All without having to think about database queries, authentification or ActivityPub, which you essentially get for free.

## Development

You can use any language to create a custom frontend. The easiest option would be forking our [official frontend](https://github.com/LemmyNet/lemmy-ui), [lemmy-lite](https://github.com/IronOxidizer/lemmy-lite), or the [lemmy-frontend-example](https://github.com/LemmyNet/lemmy-front-end-example). In any case, the principle is the same: bind to `LEMMY_EXTERNAL_HOST` (default: `localhost:8536`) and handle requests using the Lemmy API at `LEMMY_INTERNAL_HOST` (default: `lemmy:8536`). Also use `LEMMY_HTTPS` to generate links with the correct protocol.

The next step is building a Docker image from your frontend. If you forked an existing project, it should already include a `Dockerfile` and instructions to build it. Otherwise, try searching for your language on [dockerhub](https://hub.docker.com/), official images usually have build instructions in their readme. Build a Docker image with a tag, then look for the following section in `docker/dev/docker-compose.yml`:

```
  lemmy-ui:
    image: dessalines/lemmy-ui:v0.8.10
    ports:
      - "1235:1234"
    restart: always
    environment:
      - LEMMY_INTERNAL_HOST=lemmy:8536
      - LEMMY_EXTERNAL_HOST=localhost:8536
      - LEMMY_HTTPS=false
    depends_on: 
      - lemmy
```

All you need to do is replace the value for `image` with the tag of your own Docker image (and possibly the environment variables if you need different ones). Then run `./docker_update.sh`, and after compilation, your frontend will be available on `http://localhost:1235`. You can also make the same change to `docker/federation/docker-compose.yml` and run `./start-local-instances.bash` to test federation with your frontend.

## Deploy with Docker

After building the Docker image, you need to push it to a Docker registry (such as [dockerhub](https://hub.docker.com/)). Then update the `docker-compose.yml` on your server, replacing the `image` for `lemmy-ui`, just as described above. Run `docker-compose.yml`, and after a short wait, your instance will use the new frontend.

Note, if your instance is deployed with Ansible, it will override `docker-compose.yml` with every run, reverting back to the default frontend. In that case you should copy the `ansible/` folder from this project to your own repository, and adjust `docker-compose.yml` directly in the repo.

It is also possible to use multiple frontends for the same Lemmy instance, either using subdomains or subfolders. To do that, don't edit the `lemmy-ui` section in `docker-compose.yml`, but duplicate it, adjusting the name, image and port so they are distinct for each. Then edit your nginx config to pass requests to the appropriate frontend, depending on the subdomain or path.

## Translations

You can add the [lemmy-translations](https://github.com/LemmyNet/lemmy-translations) repository to your project as a [git submodule](https://git-scm.com/book/en/v2/Git-Tools-Submodules). That way you can take advantage of same translations used in the official frontend, and you will also receive new translations contributed via weblate.

## Rate limiting

Lemmy does rate limiting for many actions based on the client IP. But if you make any API calls on the server side (eg in the case of server-side rendering, or javascript pre-rendering), Lemmy will take the IP of the Docker container. Meaning that all requests come from the same IP, and get rate limited much earlier. To avoid this problem, you need to pass the headers `X-REAL-IP` and `X-FORWARDED-FOR` on to Lemmy (the headers are set by our nginx config).

Here is an example snipped for NodeJS:

```javascript
function setForwardedHeaders(
  headers: IncomingHttpHeaders
): { [key: string]: string } {
  let out = {
    host: headers.host,
  };
  if (headers['x-real-ip']) {
    out['x-real-ip'] = headers['x-real-ip'];
  }
  if (headers['x-forwarded-for']) {
    out['x-forwarded-for'] = headers['x-forwarded-for'];
  }

  return out;
}

let headers = setForwardedHeaders(req.headers);
let client = new LemmyHttp(httpUri, headers);
```