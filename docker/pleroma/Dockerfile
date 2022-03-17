# Pleroma Docker setup taken from
# https://github.com/jordemort/docker-pleroma

FROM ubuntu:20.04 AS unzip

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && \
    apt-get install -y --no-install-recommends unzip

# docker buildx will fill these in
ARG TARGETARCH=amd64
ARG TARGETVARIANT=

# Clone the release build into a temporary directory and unpack it
# We use ADD here to bust the cache if the pleroma release changes
# We use a separate layer for extraction so we don't end up with junk
# from ADD left over in the final image.
ADD https://git.pleroma.social/api/v4/projects/2/jobs/artifacts/stable/download?job=${TARGETARCH}${TARGETVARIANT:+${TARGETVARIANT}l} /tmp/pleroma.zip

RUN mkdir -p /opt/pleroma && \
    unzip /tmp/pleroma.zip -d /tmp/ && \
    mv /tmp/release/* /opt/pleroma

# Ok, really build the container now
FROM ubuntu:20.04 AS pleroma

ENV DEBIAN_FRONTEND=noninteractive

ARG SOAPBOXVERSION=1.2.3

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates curl dumb-init ffmpeg gnupg imagemagick libimage-exiftool-perl libmagic-dev libncurses5 locales postgresql-client-12 unzip && \
    apt-get clean

RUN echo 'en_US.UTF-8 UTF-8' > /etc/locale.gen && \
    locale-gen

ENV LANG en_US.UTF-8
ENV LANGUAGE en_US:en
ENV LC_ALL en_US.UTF-8

RUN mkdir -p /etc/pleroma /var/lib/pleroma/static /var/lib/pleroma/uploads && \
    adduser --system --shell /bin/false --home /opt/pleroma --group pleroma && \
    chown -vR pleroma /etc/pleroma /var/lib/pleroma

COPY --chown=pleroma:pleroma --from=unzip /opt/pleroma/ /opt/pleroma/

VOLUME [ "/etc/pleroma", "/var/lib/pleroma/uploads", "/var/lib/pleroma/static" ]

ADD https://gitlab.com/soapbox-pub/soapbox-fe/-/jobs/artifacts/v${SOAPBOXVERSION}/download?job=build-production /tmp/soapbox-fe.zip
RUN chown pleroma /tmp/soapbox-fe.zip

USER pleroma

COPY run-pleroma.sh /opt/pleroma/bin/

ENTRYPOINT [ "/usr/bin/dumb-init" ]

WORKDIR /opt/pleroma

ENV PATH=/opt/pleroma/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin
ENV PLEROMA_CONFIG_PATH=/etc/pleroma/config.exs

EXPOSE 4000

STOPSIGNAL SIGTERM

HEALTHCHECK \
    --start-period=2m \
    --interval=5m \
    CMD curl --fail http://localhost:4000/api/v1/instance || exit 1

CMD [ "run-pleroma.sh" ]
