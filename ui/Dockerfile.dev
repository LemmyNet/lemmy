# Setup env
FROM node:10-alpine
RUN mkdir -p /opt/lemmy/ui--dev
WORKDIR /opt/lemmy/ui--dev
# Install deps
COPY package.json .
COPY yarn.lock .
RUN npm install
# Add app
COPY . .
# Run app
CMD ["npm", "start"]
