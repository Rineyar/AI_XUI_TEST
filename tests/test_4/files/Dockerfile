FROM node:22-slim

WORKDIR /app
ENV OPEN_BROWSER=0

COPY package.json package-lock.json ./
RUN npm ci --omit=dev

COPY server.js LICENSE ./
COPY data ./data
COPY public ./public

USER node
EXPOSE 3000
CMD ["node", "server.js"]
