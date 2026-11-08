FROM node:18-alpine

ENV NODE_ENV=production

WORKDIR /app

COPY backend/package*.json ./
RUN npm install --omit=dev

COPY backend/src ./src

ENTRYPOINT ["node"]
CMD ["src/index.js"]
