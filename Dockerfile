FROM node:18-alpine

ENV NODE_ENV=production

WORKDIR /app

RUN apk add --upgrade --no-cache python3 make g++
RUN npm install -g pnpm

COPY backend/package*.json backend/pnpm-lock.yaml* ./
RUN pnpm install --prod

COPY backend/src ./src

ENTRYPOINT ["node"]
CMD ["src/index.js"]

