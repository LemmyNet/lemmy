# Setup env
FROM node:10-alpine AS build
RUN mkdir -p /opt/lemmy/ui--prod
WORKDIR /opt/lemmy/ui--prod
# Install deps
COPY package.json .
COPY yarn.lock .
RUN npm install
# Add app
COPY . .
# Build app
RUN npm run build

# Setup env
FROM node:10-alpine
RUN mkdir -p /opt/lemmy/ui--prod
WORKDIR /opt/lemmy/ui--prod
RUN npm install serve
# Add app
COPY --from=build /opt/lemmy/ui--prod/dist .
# Run app
CMD ["/opt/lemmy/ui--prod/node_modules/.bin/serve", "."]
