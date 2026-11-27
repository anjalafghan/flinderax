FROM oven/bun:1
WORKDIR /app
COPY mcp/package.json mcp/bun.lock ./
RUN bun install --production --frozen-lockfile
COPY mcp/src ./src
ENV NODE_ENV=production
CMD ["bun", "run", "src/http.ts"]
